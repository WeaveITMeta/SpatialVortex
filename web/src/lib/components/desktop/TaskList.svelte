<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let tasks: Task[] = [];
  
  interface Task {
    id: string;
    title: string;
    description?: string;
    status: 'pending' | 'in_progress' | 'completed' | 'blocked';
    priority: 'low' | 'medium' | 'high';
    created_at: Date;
    updated_at: Date;
  }
  
  function toggleTaskStatus(task: Task) {
    const newStatus = task.status === 'completed' ? 'pending' : 'completed';
    dispatch('statusChange', { id: task.id, status: newStatus });
  }
  
  function getStatusIcon(status: string): string {
    switch (status) {
      case 'completed': return '✅';
      case 'in_progress': return '⏳';
      case 'blocked': return '🚫';
      default: return '⭕';
    }
  }
  
  function getPriorityColor(priority: string): string {
    switch (priority) {
      case 'high': return '#ef4444';
      case 'medium': return '#f59e0b';
      case 'low': return '#22c55e';
      default: return '#71717a';
    }
  }
  
  // Group tasks by status
  $: tasksByStatus = {
    in_progress: tasks.filter(t => t.status === 'in_progress'),
    pending: tasks.filter(t => t.status === 'pending'),
    completed: tasks.filter(t => t.status === 'completed'),
    blocked: tasks.filter(t => t.status === 'blocked'),
  };
  
  let showCompleted = false;
</script>

<div class="task-list">
  <div class="task-header">
    <div class="task-header-left">
      <h3 class="task-title">🧠 Task Tracker</h3>
      <div class="task-stats">
        <span class="stat-badge" title="In Progress">
          ⏳ {tasksByStatus.in_progress.length}
        </span>
        <span class="stat-badge" title="Pending">
          ⭕ {tasksByStatus.pending.length}
        </span>
        <span class="stat-badge" title="Completed">
          ✅ {tasksByStatus.completed.length}
        </span>
      </div>
    </div>
  </div>
  
  {#if tasks.length === 0}
    <div class="empty-state">
      <div class="empty-icon">📝</div>
      <p>No active tasks</p>
      <p class="empty-hint">Tasks will appear here automatically as the AI works</p>
    </div>
  {:else}
    <!-- In Progress Tasks -->
    {#if tasksByStatus.in_progress.length > 0}
      <div class="task-section">
        <h4 class="section-title">⏳ In Progress</h4>
        {#each tasksByStatus.in_progress as task}
          <div class="task-item in-progress" on:click={() => toggleTaskStatus(task)}>
            <div class="task-icon">{getStatusIcon(task.status)}</div>
            <div class="task-content">
              <div class="task-item-title">{task.title}</div>
              {#if task.description}
                <div class="task-description">{task.description}</div>
              {/if}
            </div>
            <div 
              class="priority-badge" 
              style="background: {getPriorityColor(task.priority)};"
              title="{task.priority} priority"
            ></div>
          </div>
        {/each}
      </div>
    {/if}
    
    <!-- Pending Tasks -->
    {#if tasksByStatus.pending.length > 0}
      <div class="task-section">
        <h4 class="section-title">⭕ Pending</h4>
        {#each tasksByStatus.pending as task}
          <div class="task-item pending" on:click={() => toggleTaskStatus(task)}>
            <div class="task-icon">{getStatusIcon(task.status)}</div>
            <div class="task-content">
              <div class="task-item-title">{task.title}</div>
              {#if task.description}
                <div class="task-description">{task.description}</div>
              {/if}
            </div>
            <div 
              class="priority-badge" 
              style="background: {getPriorityColor(task.priority)};"
            ></div>
          </div>
        {/each}
      </div>
    {/if}
    
    <!-- Blocked Tasks -->
    {#if tasksByStatus.blocked.length > 0}
      <div class="task-section">
        <h4 class="section-title">🚫 Blocked</h4>
        {#each tasksByStatus.blocked as task}
          <div class="task-item blocked">
            <div class="task-icon">{getStatusIcon(task.status)}</div>
            <div class="task-content">
              <div class="task-item-title">{task.title}</div>
              {#if task.description}
                <div class="task-description">{task.description}</div>
              {/if}
            </div>
            <div 
              class="priority-badge" 
              style="background: {getPriorityColor(task.priority)};"
            ></div>
          </div>
        {/each}
      </div>
    {/if}
    
    <!-- Completed Tasks (Collapsible) -->
    {#if tasksByStatus.completed.length > 0}
      <div class="task-section">
        <h4 
          class="section-title clickable" 
          on:click={() => showCompleted = !showCompleted}
        >
          ✅ Completed ({tasksByStatus.completed.length})
          <span class="toggle-icon">{showCompleted ? '▼' : '▶'}</span>
        </h4>
        {#if showCompleted}
          {#each tasksByStatus.completed as task}
            <div class="task-item completed" on:click={() => toggleTaskStatus(task)}>
              <div class="task-icon">{getStatusIcon(task.status)}</div>
              <div class="task-content">
                <div class="task-item-title">{task.title}</div>
                {#if task.description}
                  <div class="task-description">{task.description}</div>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .task-list {
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 1rem;
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .task-header {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .task-header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  
  .task-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .task-stats {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  
  .stat-badge {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    font-size: 0.875rem;
    color: #d4d4d4;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    text-align: center;
    color: #71717a;
  }
  
  .empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }
  
  .empty-hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
  
  .task-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .section-title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: #a1a1aa;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .section-title.clickable {
    cursor: pointer;
    padding: 0.5rem;
    margin: -0.5rem;
    border-radius: 6px;
    transition: background 0.2s;
  }
  
  .section-title.clickable:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .toggle-icon {
    margin-left: auto;
    font-size: 0.75rem;
  }
  
  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .task-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.2);
  }
  
  .task-item.in-progress {
    border-left: 3px solid #3b82f6;
  }
  
  .task-item.pending {
    border-left: 3px solid #71717a;
  }
  
  .task-item.completed {
    opacity: 0.6;
  }
  
  .task-item.completed .task-item-title {
    text-decoration: line-through;
  }
  
  .task-item.blocked {
    border-left: 3px solid #ef4444;
    background: rgba(239, 68, 68, 0.05);
  }
  
  .task-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }
  
  .task-content {
    flex: 1;
    min-width: 0;
  }
  
  .task-item-title {
    font-weight: 500;
    color: #e4e4e7;
    margin-bottom: 0.25rem;
  }
  
  .task-description {
    font-size: 0.875rem;
    color: #a1a1aa;
    line-height: 1.4;
  }
  
  .priority-badge {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 0.375rem;
  }
</style>
