<script lang="ts">
  export let headers: string[];
  export let rows: string[][];
  
  let copied = false;
  
  function copyAsMarkdown() {
    // Build markdown table
    const headerRow = '| ' + headers.join(' | ') + ' |';
    const separator = '| ' + headers.map(() => '---').join(' | ') + ' |';
    const dataRows = rows.map(row => '| ' + row.join(' | ') + ' |');
    
    const markdown = [headerRow, separator, ...dataRows].join('\n');
    
    navigator.clipboard.writeText(markdown);
    copied = true;
    setTimeout(() => copied = false, 2000);
  }
</script>

<div class="table-container">
  <div class="table-header">
    <button class="copy-btn" on:click={copyAsMarkdown} title="Copy as markdown">
      {#if copied}
        ✓ Copied!
      {:else}
        📋 Copy Table
      {/if}
    </button>
  </div>
  
  <div class="table-wrapper">
    <table>
      <thead>
        <tr>
          {#each headers as header}
            <th>{header}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row}
          <tr>
            {#each row as cell}
              <td>{cell}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .table-container {
    margin: 1rem 0;
    border-radius: 8px;
    overflow: hidden;
    background: #181825;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }
  
  .table-header {
    background: #1e1e2e;
    padding: 0.5rem 1rem;
    display: flex;
    justify-content: flex-end;
    border-bottom: 1px solid #313244;
  }
  
  .copy-btn {
    background: #313244;
    border: none;
    color: #cdd6f4;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s ease;
  }
  
  .copy-btn:hover {
    background: #45475a;
    transform: translateY(-1px);
  }
  
  .copy-btn:active {
    transform: translateY(0);
  }
  
  .table-wrapper {
    overflow-x: auto;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
  }
  
  thead {
    background: #313244;
  }
  
  th {
    color: #60a5fa;
    padding: 1rem;
    text-align: left;
    font-weight: 600;
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 2px solid #45475a;
  }
  
  td {
    padding: 1rem;
    border-bottom: 1px solid #313244;
    color: #cdd6f4;
  }
  
  tbody tr {
    transition: background 0.2s ease;
  }
  
  tbody tr:hover {
    background: #1e1e2e;
  }
  
  tbody tr:last-child td {
    border-bottom: none;
  }
  
  /* Alternating row colors */
  tbody tr:nth-child(even) {
    background: rgba(49, 50, 68, 0.3);
  }
  
  tbody tr:nth-child(even):hover {
    background: #1e1e2e;
  }
</style>
