-- Session Memory Schema
-- Stores conversation sessions with messages and metadata

-- Conversation sessions table
CREATE TABLE IF NOT EXISTS conversation_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    summary TEXT,
    user_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    message_count INTEGER DEFAULT 0,
    last_message_at TIMESTAMP WITH TIME ZONE,
    is_archived BOOLEAN DEFAULT FALSE,
    tags TEXT[],
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Session messages table
CREATE TABLE IF NOT EXISTS session_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES conversation_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'user' or 'assistant'
    content TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    token_count INTEGER,
    model VARCHAR(100),
    CONSTRAINT valid_role CHECK (role IN ('user', 'assistant', 'system'))
);

-- Message embeddings for semantic search
CREATE TABLE IF NOT EXISTS message_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES session_messages(id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES conversation_sessions(id) ON DELETE CASCADE,
    embedding vector(384), -- For semantic search
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON conversation_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON conversation_sessions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON conversation_sessions(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_archived ON conversation_sessions(is_archived);
CREATE INDEX IF NOT EXISTS idx_messages_session_id ON session_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON session_messages(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_embeddings_session_id ON message_embeddings(session_id);

-- Full-text search indexes
CREATE INDEX IF NOT EXISTS idx_sessions_title_search ON conversation_sessions USING gin(to_tsvector('english', title));
CREATE INDEX IF NOT EXISTS idx_sessions_summary_search ON conversation_sessions USING gin(to_tsvector('english', summary));
CREATE INDEX IF NOT EXISTS idx_messages_content_search ON session_messages USING gin(to_tsvector('english', content));

-- Function to update session timestamp and message count
CREATE OR REPLACE FUNCTION update_session_metadata()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversation_sessions
    SET 
        updated_at = NOW(),
        last_message_at = NEW.timestamp,
        message_count = (
            SELECT COUNT(*) 
            FROM session_messages 
            WHERE session_id = NEW.session_id
        )
    WHERE id = NEW.session_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update session metadata
DROP TRIGGER IF EXISTS update_session_on_message ON session_messages;
CREATE TRIGGER update_session_on_message
    AFTER INSERT ON session_messages
    FOR EACH ROW
    EXECUTE FUNCTION update_session_metadata();

-- Function to auto-generate session title from first message
CREATE OR REPLACE FUNCTION generate_session_title()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.title = 'New Conversation' AND NEW.message_count >= 1 THEN
        UPDATE conversation_sessions
        SET title = SUBSTRING(
            (SELECT content FROM session_messages 
             WHERE session_id = NEW.id 
             AND role = 'user'
             ORDER BY timestamp ASC 
             LIMIT 1),
            1, 100
        )
        WHERE id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-generate title
DROP TRIGGER IF EXISTS auto_generate_title ON conversation_sessions;
CREATE TRIGGER auto_generate_title
    AFTER UPDATE OF message_count ON conversation_sessions
    FOR EACH ROW
    WHEN (NEW.message_count >= 1)
    EXECUTE FUNCTION generate_session_title();

-- Sample data for testing (optional)
INSERT INTO conversation_sessions (title, summary, user_id, tags) VALUES
    ('React Performance Optimization', 'Discussion about React.memo and useMemo', 'demo_user', ARRAY['react', 'performance']),
    ('PostgreSQL Indexing Strategies', 'Deep dive into B-tree vs GiST indexes', 'demo_user', ARRAY['database', 'postgresql'])
ON CONFLICT DO NOTHING;
