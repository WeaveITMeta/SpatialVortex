// Mock API for SpatialVortex Chat Interface
// Run with: bun mock-api.js

import { serve } from '@hono/node-server';
import { Hono } from 'hono';
import { cors } from 'hono/cors';

const app = new Hono();

// Enable CORS
app.use('/*', cors());

// Position names
const positionNames = {
  0: 'Divine Source',
  1: 'New Beginning',
  2: 'Duality',
  3: 'Sacred Trinity',
  4: 'Foundation',
  5: 'Transformation',
  6: 'Sacred Balance',
  7: 'Wisdom',
  8: 'Potential',
  9: 'Sacred Completion',
};

// Mock response generator
function generateMockResponse(message) {
  // Analyze message for ELP hints
  const lowercaseMsg = message.toLowerCase();
  
  let ethos = 7 + Math.random() * 3;
  let logos = 6 + Math.random() * 4;
  let pathos = 5 + Math.random() * 5;
  
  // Adjust based on keywords
  if (lowercaseMsg.includes('love') || lowercaseMsg.includes('ethics') || lowercaseMsg.includes('moral')) {
    ethos += 2;
  }
  if (lowercaseMsg.includes('logic') || lowercaseMsg.includes('reason') || lowercaseMsg.includes('think')) {
    logos += 2;
  }
  if (lowercaseMsg.includes('feel') || lowercaseMsg.includes('emotion') || lowercaseMsg.includes('heart')) {
    pathos += 2;
  }
  
  // Special cases for sacred numbers
  if (lowercaseMsg.includes('3') || lowercaseMsg.includes('6') || lowercaseMsg.includes('9')) {
    ethos = 9 + Math.random() * 2;
    logos = 9 + Math.random() * 2;
    pathos = 9 + Math.random() * 2;
  }
  
  // Normalize to range
  ethos = Math.min(13, Math.max(0, ethos));
  logos = Math.min(13, Math.max(0, logos));
  pathos = Math.min(13, Math.max(0, pathos));
  
  // Calculate confidence
  const confidence = 0.6 + Math.random() * 0.3;
  
  // Determine flux position
  let position;
  const total = ethos + logos + pathos;
  const eNorm = ethos / total;
  const lNorm = logos / total;
  const pNorm = pathos / total;
  
  // Sacred positions if high confidence
  if (confidence > 0.75) {
    if (eNorm > lNorm && eNorm > pNorm) position = 3;
    else if (pNorm > eNorm && pNorm > lNorm) position = 6;
    else if (lNorm > eNorm && lNorm > pNorm) position = 9;
    else position = 0;
  } else {
    // Regular positions
    if (eNorm > lNorm && eNorm > pNorm) position = 1;
    else if (pNorm > eNorm && pNorm > lNorm) position = 5;
    else if (lNorm > eNorm && lNorm > pNorm) position = 8;
    else position = 4;
  }
  
  // Generate contextual response
  let response = `You asked: "${message}"\n\n`;
  
  response += `Sacred Geometry Analysis:\n`;
  response += `• Confidence: ${(confidence * 100).toFixed(0)}% (${confidence > 0.7 ? 'High certainty' : 'Moderate certainty'})\n`;
  response += `• Flux Position: ${position} - ${positionNames[position]}\n`;
  
  if ([3, 6, 9].includes(position)) {
    response += `• ✨ This is a SACRED position! High geometric significance.\n`;
  }
  
  response += `\nELP Channel Breakdown:\n`;
  response += `• Ethos (Ethics): ${ethos.toFixed(1)}/13 - ${ethos > 9 ? 'Strong' : ethos > 6 ? 'Moderate' : 'Low'}\n`;
  response += `• Logos (Logic): ${logos.toFixed(1)}/13 - ${logos > 9 ? 'Strong' : logos > 6 ? 'Moderate' : 'Low'}\n`;
  response += `• Pathos (Emotion): ${pathos.toFixed(1)}/13 - ${pathos > 9 ? 'Strong' : pathos > 6 ? 'Moderate' : 'Low'}\n`;
  
  const dominant = ethos > logos && ethos > pathos ? 'Ethos' : 
                   logos > pathos ? 'Logos' : 'Pathos';
  response += `\nDominant Channel: ${dominant}\n`;
  
  response += `\n⚡ This is a MOCK response for testing. Real AI integration coming soon!`;
  
  return {
    response,
    elp_values: {
      ethos,
      logos,
      pathos,
    },
    confidence: confidence,
    flux_position: position,
    processing_time_ms: 150,
  };
}

// Chat endpoint
app.post('/api/v1/chat/text', async (c) => {
  try {
    const body = await c.req.json();
    const { message, user_id } = body;
    
    if (!message) {
      return c.json({ error: 'Message is required' }, 400);
    }
    
    // Simulate processing time
    await new Promise(resolve => setTimeout(resolve, 500));
    
    const response = generateMockResponse(message);
    
    console.log(`[${new Date().toISOString()}] Chat request from ${user_id}: "${message.substring(0, 50)}..."`);
    console.log(`  → ELP: E=${response.elp_values.ethos.toFixed(1)} L=${response.elp_values.logos.toFixed(1)} P=${response.elp_values.pathos.toFixed(1)}`);
    console.log(`  → Confidence: ${(response.confidence * 100).toFixed(0)}% | Position: ${response.flux_position}`);
    
    return c.json(response);
  } catch (error) {
    console.error('Error processing chat request:', error);
    return c.json({ error: 'Internal server error' }, 500);
  }
});

// Health check
app.get('/health', (c) => {
  return c.json({
    status: 'ok',
    service: 'SpatialVortex Mock API',
    version: '1.0.0',
    timestamp: new Date().toISOString(),
  });
});

// Root
app.get('/', (c) => {
  return c.json({
    message: '🌀 SpatialVortex Mock API',
    endpoints: {
      'POST /api/v1/chat/text': 'Chat with text input',
      'GET /health': 'Health check',
    },
    note: 'This is a mock API for frontend development. Real backend with ONNX models coming soon!',
  });
});

// Start server
const port = 8080;
serve({
  fetch: app.fetch,
  port,
});

console.log('');
console.log('🌀 SpatialVortex Mock API Server');
console.log('================================');
console.log('');
console.log(`✅ Server running at http://localhost:${port}`);
console.log('');
console.log('Endpoints:');
console.log(`  • POST http://localhost:${port}/api/v1/chat/text`);
console.log(`  • GET  http://localhost:${port}/health`);
console.log('');
console.log('⚡ Ready to receive chat requests!');
console.log('📝 Testing: Open http://localhost:5173 in your browser');
console.log('');
