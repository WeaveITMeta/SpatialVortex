# Remove large benchmark files from git history
Write-Host "Removing large files from git history..." -ForegroundColor Yellow

# This will rewrite all commits to remove the benchmarks/data directory
git filter-branch --force --index-filter `
  "git rm -rf --cached --ignore-unmatch benchmarks/data" `
  --prune-empty --tag-name-filter cat -- --all

Write-Host "Cleaning up..." -ForegroundColor Yellow
# Clean up backup refs
Remove-Item -Path .git/refs/original -Recurse -Force -ErrorAction SilentlyContinue

# Expire reflog
git reflog expire --expire=now --all

# Garbage collect
git gc --prune=now --aggressive

Write-Host "Done! Large files removed from history." -ForegroundColor Green
Write-Host "Now you can push with: git push origin main --force" -ForegroundColor Cyan
