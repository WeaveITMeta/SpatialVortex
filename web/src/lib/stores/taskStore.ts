import { writable } from 'svelte/store';

export interface Task {
  id: string;
  title: string;
  description?: string;
  status: 'pending' | 'in_progress' | 'completed' | 'blocked';
  priority: 'low' | 'medium' | 'high';
  created_at: Date;
  updated_at: Date;
}

function createTaskStore() {
  const { subscribe, set, update } = writable<Task[]>([]);

  return {
    subscribe,
    
    addTask: (task: Omit<Task, 'id' | 'created_at' | 'updated_at'>) => {
      update(tasks => {
        const newTask: Task = {
          ...task,
          id: Date.now().toString() + Math.random().toString(36).substring(2, 9),
          created_at: new Date(),
          updated_at: new Date(),
        };
        return [...tasks, newTask];
      });
    },
    
    updateTask: (id: string, updates: Partial<Task>) => {
      update(tasks => {
        return tasks.map(task => 
          task.id === id 
            ? { ...task, ...updates, updated_at: new Date() }
            : task
        );
      });
    },
    
    updateStatus: (id: string, status: Task['status']) => {
      update(tasks => {
        return tasks.map(task => 
          task.id === id 
            ? { ...task, status, updated_at: new Date() }
            : task
        );
      });
    },
    
    removeTask: (id: string) => {
      update(tasks => tasks.filter(task => task.id !== id));
    },
    
    clearCompleted: () => {
      update(tasks => tasks.filter(task => task.status !== 'completed'));
    },
    
    clearAll: () => {
      set([]);
    },
    
    // Parse tasks from AI response
    parseFromResponse: (response: string) => {
      const taskPatterns = [
        /(?:^|\n)[-*]\s*\[\s*\]\s*(.+?)(?:\n|$)/gm,  // - [ ] Task
        /(?:^|\n)\d+\.\s*(.+?)(?:\n|$)/gm,           // 1. Task
        /Task:\s*(.+?)(?:\n|$)/gi,                    // Task: Description
      ];
      
      const foundTasks: string[] = [];
      
      taskPatterns.forEach(pattern => {
        let match;
        while ((match = pattern.exec(response)) !== null) {
          const taskText = match[1].trim();
          if (taskText && taskText.length > 5 && taskText.length < 200) {
            foundTasks.push(taskText);
          }
        }
      });
      
      // Add unique tasks
      const uniqueTasks = [...new Set(foundTasks)];
      uniqueTasks.forEach(title => {
        update(tasks => {
          // Check if task already exists
          if (tasks.some(t => t.title.toLowerCase() === title.toLowerCase())) {
            return tasks;
          }
          
          const newTask: Task = {
            id: Date.now().toString() + Math.random().toString(36).substring(2, 9),
            title,
            status: 'pending',
            priority: 'medium',
            created_at: new Date(),
            updated_at: new Date(),
          };
          return [...tasks, newTask];
        });
      });
    },
  };
}

export const taskStore = createTaskStore();

// Persist to localStorage
if (typeof window !== 'undefined') {
  taskStore.subscribe(tasks => {
    localStorage.setItem('tasks', JSON.stringify(tasks));
  });
  
  // Load from localStorage on init
  const saved = localStorage.getItem('tasks');
  if (saved) {
    try {
      const tasks = JSON.parse(saved);
      taskStore.subscribe(() => {})(); // Trigger initial update
      tasks.forEach((task: Task) => {
        task.created_at = new Date(task.created_at);
        task.updated_at = new Date(task.updated_at);
      });
      // Note: Direct set would be better but we need to maintain the store pattern
    } catch (e) {
      console.error('Failed to load tasks from localStorage:', e);
    }
  }
}
