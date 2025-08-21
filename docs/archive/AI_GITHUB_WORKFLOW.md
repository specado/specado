# AI GitHub Workflow Guidelines

This document outlines the standardized approach for AI assistants working with GitHub issues, PRs, and project management in the Specado project.

## 🚀 Quick Start Checklist for Every GitHub Operation

Before ANY GitHub operation:
1. [ ] Search for duplicates: `gh issue list --search "topic" --state all`
2. [ ] Check labels exist: `gh label list --limit 100`
3. [ ] Use single quotes in gh commands
4. [ ] Have evidence ready when closing issues

## 🚨 Critical Rules - Always Follow

1. **Check Before Creating**:
   - Run `gh issue list --search "topic" --state all` before creating issues
   - Run `gh label list --limit 100` before creating labels
   - Never create duplicate issues or labels

2. **Label Discipline**:
   - Use ONLY existing labels when possible
   - Never create variants (e.g., `high-priority` when `priority:high` exists)
   - Follow existing patterns (`category:value` or single word)

3. **GitHub CLI Syntax**:
   - Always use single quotes in `gh` commands (double quotes can be buggy)
   - Example: `gh issue close 35 --comment 'message'` ✅

4. **Issue Hygiene**:
   - Close issues with evidence of completion
   - Update dependent issues when completing work
   - Cross-reference liberally to maintain context

## Issue Management

### Before Creating Issues
**ALWAYS check for duplicates first:**
```bash
# Search existing issues (open and closed)
gh issue list --search "config management" --state all
gh issue list --search "credentials" --state all

# Check by label
gh issue list --label "task" --state all

# Check specific epic's issues
gh issue view 5  # View epic to see related issues
```

### Creating Issues
Only create new issues after confirming no duplicates exist. Use structured templates with consistent sections:

```markdown
## 📋 Task Overview
[Brief description of what needs to be done]

## 🎯 Acceptance Criteria
- [ ] Specific, measurable outcomes
- [ ] Clear completion requirements
- [ ] Testable conditions

## 📊 Technical Details
### Implementation Approach
[High-level approach without being prescriptive]

### Dependencies
- Uses: #[issue-number] (description)
- Related to: #[issue-number] (description)

### API Design
```language
// Minimal API sketch if relevant
```

## ⚠️ Risks & Considerations
- [Potential challenges]
- [Things to watch out for]

## 🧪 Testing Requirements
- Unit: [what to unit test]
- Integration: [integration test scope]
- E2E: [end-to-end test needs]

## 📚 Documentation Requirements
- [ ] What documentation needs updating
- [ ] New documentation needed

## 🔗 References
- Epic: #[epic-number] (Epic Name)
- Related: #[issue-numbers]

## ⏱️ Estimates
- **Effort**: XS/S/M/L/XL
- **Time**: X-Y hours
- **Complexity**: Low/Medium/High
```

### Closing Issues
When closing an issue via commit or manually:

1. **In commit message**: Reference the issue
   ```
   feat(module): implement feature (#35)
   
   - Implementation details
   - What was done
   
   Closes #35
   ```

2. **Via GitHub CLI**: Include implementation summary
   ```bash
   gh issue close 35 --comment 'Implemented in commit [sha]. 
   
   The implementation includes:
   - ✅ [Completed criteria 1]
   - ✅ [Completed criteria 2]
   - ✅ [Additional work done]'
   ```

### Commenting on Issues

#### Progress Updates
```markdown
**Update**: [Dependency/Blocker] has been resolved.

[Relevant code example or usage]

[What's now possible]

Ready to proceed with [next step].
```

#### Completion Comments
```markdown
Implemented in commit [sha].

The implementation includes:
- ✅ [Each acceptance criterion, marked complete]
- ✅ [Any additional work done]

[Any relevant notes about the implementation]
```

#### Cross-Reference Updates
When completing a dependency, update all dependent issues:
```markdown
**Update**: Issue #[number] ([Name]) has been completed. 

[What this enables]

[Usage example if relevant]

Ready to proceed with [this issue's work].
```

## Pull Request Management

### PR Descriptions
Use structured templates:

```markdown
## Summary
[1-3 bullet points of what this PR does]

## Changes
- [Detailed list of changes]
- [File modifications]
- [New features/fixes]

## Testing
- [x] Unit tests added/updated
- [x] Integration tests pass
- [x] Manual testing completed

## Related Issues
Closes #[issue-number]
Related to #[issue-numbers]

## Checklist
- [x] Code follows project style
- [x] Tests pass
- [x] Documentation updated
- [x] No sensitive data exposed
```

### PR Comments
For feedback or review responses:
```markdown
Thanks for the feedback! Changes made:

1. **[Feedback point]**: [How it was addressed]
2. **[Feedback point]**: [How it was addressed]

[Any questions or clarifications needed]
```

## Commit Message Standards

### Format
```
type(scope): short description

- Detailed change 1
- Detailed change 2
- Additional context

[Footer with issue references]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring
- `test`: Test additions/changes
- `docs`: Documentation only
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

### Examples
```
feat(spec-runner): implement credentials and config management (#35)

- Add RunnerConfig struct for centralized configuration
- Support environment variables, JSON, and TOML config files  
- Implement proper precedence: explicit > file > env > defaults
- Add automatic redaction of sensitive values in logs
- Include comprehensive tests for all configuration sources

Closes #35
```

### CI/CD and Release Commits
```
feat(ci): update release workflow

- Replace deprecated actions
- Fix PyPI publishing dependencies  
- Update to latest macOS runners

Never include "Generated with Claude" or author attributions
```

## Issue Linking and References

### In Issues
- **Dependencies**: `Uses: #22 (Retry Logic), #23 (Token Accounting)`
- **Related**: `Related to: #5 (Epic), #21 (Similar implementation)`
- **Blocks**: `Blocks: #30, #31 (Cannot proceed until this is done)`
- **Blocked by**: `Blocked by: #35 (Waiting on configuration)`

### In Code Comments
```rust
// TODO(issue-20): Implement streaming support
// See issue #35 for configuration details
// Addresses feedback from #22
```

### In Commits
- `Closes #N` - Automatically closes issue when merged
- `Fixes #N` - Same as closes
- `Resolves #N` - Same as closes
- `See #N` - References without closing
- `Related to #N` - References without closing

## Epic and Milestone Management

### Epic Structure
Epics should be clean and focused:
```markdown
# Epic: [Name]

[Brief description of the epic's goal]

**Sprint**: [Sprint range]  
**Priority**: [Critical/High/Medium/Low]  
**Risk**: [Risk level and brief explanation]

## Success Criteria
- [Clear, measurable outcomes]
- [What defines completion]
```

### Parent-Child Relationships (Sub-Issues)
Use GitHub's native sub-issue feature instead of task lists for proper tracking:

#### Important: Parent-Child vs References
- Use parent-child for: Tasks that are part of an epic
- Use references for: Related but independent issues
- Use "Blocks/Blocked by" for: Dependencies between peers

#### Creating Parent-Child Relationships
```bash
# First, get the issue IDs using GraphQL
gh api graphql -f query='
{
  repository(owner: "specado", name: "specado") {
    parent: issue(number: 42) { id }
    child: issue(number: 47) { id }
  }
}'

# Then create the parent-child relationship
gh api graphql \
  -H 'GraphQL-Features: sub_issues' \
  -f query='
mutation {
  addSubIssue(input: {
    issueId: "PARENT_ID"
    subIssueId: "CHILD_ID"
  }) {
    issue { number }
    subIssue { number }
  }
}'
```

#### Batch Creating Relationships
```bash
# Create multiple parent-child relationships in one mutation
gh api graphql \
  -H 'GraphQL-Features: sub_issues' \
  -f query='
mutation {
  add1: addSubIssue(input: {
    issueId: "EPIC_ID"
    subIssueId: "TASK1_ID"
  }) {
    issue { number }
    subIssue { number }
  }
  add2: addSubIssue(input: {
    issueId: "EPIC_ID"
    subIssueId: "TASK2_ID"
  }) {
    issue { number }
    subIssue { number }
  }
}'
```

### Verifying Relationships
```bash
# Check if a child has a parent
gh api graphql -f query='
{
  repository(owner: "specado", name: "specado") {
    issue(number: 47) {
      number
      title
      parent {
        number
        title
      }
    }
  }
}'

# Check an epic's sub-issues
gh api graphql -f query='
{
  repository(owner: "specado", name: "specado") {
    issue(number: 42) {
      number
      title
      subIssues(first: 10) {
        totalCount
        nodes {
          number
          title
        }
      }
    }
  }
}'
```

## Status Tracking

### Issue States
- **OPEN**: Work not started or in progress
- **CLOSED**: Work completed and verified
- Use labels for more granular status:
  - `in-progress`: Actively being worked on
  - `blocked`: Waiting on dependency
  - `ready-for-review`: Implementation complete
  - `needs-revision`: Requires changes

### Progress Comments
Regular updates on long-running issues:
```markdown
**Progress Update**:
- ✅ Configuration struct implemented
- ✅ Environment variable loading
- 🔄 Working on file loading
- ⏳ Tests pending
```

## Best Practices

### Do's
- ✅ Keep issue descriptions focused and actionable
- ✅ Use parent-child relationships instead of task lists for tracking
- ✅ Update dependent issues when completing work
- ✅ Use checkboxes for trackable progress
- ✅ Include code examples in updates
- ✅ Reference commits and PRs
- ✅ Close issues with summary of work done
- ✅ Keep epics clean - no redundant information

### Don'ts
- ❌ Leave issues open after work is complete
- ❌ Create duplicate issues without checking
- ❌ Make breaking changes without noting in PR
- ❌ Close issues without explanation
- ❌ Forget to update related issues
- ❌ Add unnecessary comments that clutter issues
- ❌ Use task lists when parent-child relationships are more appropriate

### AI-Specific Guidelines
1. **Always use single quotes in gh commands** (double quotes can be buggy)
2. **Include implementation evidence** when closing issues
3. **Use GraphQL for advanced operations** (parent-child, project fields)
4. **Clean up issues** - remove redundant comments and task lists
5. **Provide usage examples** in completion comments
6. **Check issue state** before commenting/closing
7. **Use batch operations** when updating multiple items
8. **Never use scripts for batch operations** - Apply changes directly with individual gh commands or use GraphQL mutations with multiple aliases in a single request
9. **Use GraphQL node IDs** for parent-child relationships, not issue numbers

### Issue Cleanup Best Practices
```bash
# Remove comments programmatically
gh api repos/OWNER/REPO/issues/ISSUE_NUMBER/comments --jq '.[].id' | while read comment_id; do
  # Check comment content before deleting
  comment_body=$(gh api repos/OWNER/REPO/issues/comments/$comment_id --jq '.body')
  if echo "$comment_body" | grep -q "PATTERN_TO_REMOVE"; then
    gh api -X DELETE repos/OWNER/REPO/issues/comments/$comment_id
  fi
done

# Clean up epic descriptions - remove task lists after creating parent-child relationships
gh issue edit EPIC_NUMBER --body 'Clean, focused epic description without task lists'
```

## Label Management

### Label Guidelines

#### Before Creating Labels
**ALWAYS check existing labels first:**
```bash
# List all labels in the repository
gh label list --limit 100

# Search for specific labels
gh label list --search "priority"
gh label list --search "task"
```

#### Label Categories
Use existing label patterns - DO NOT create variants:

**Priority Labels** (use existing):
- `priority:critical` - Immediate attention required
- `priority:high` - Should be addressed soon
- `priority:medium` - Normal priority
- `priority:low` - Can wait

**Type Labels** (use existing):
- `epic` - Large feature or initiative
- `task` - Standard work item
- `bug` - Something broken
- `documentation` - Docs only
- `infrastructure` - CI/CD, build, deploy

**Status Labels** (use existing if present):
- `blocked` - Cannot proceed
- `in-progress` - Active work
- `ready-for-review` - Awaiting review
- `needs-revision` - Changes requested

#### Label Anti-Patterns - NEVER CREATE THESE
**Check `gh label list` output first. If any of these patterns exist, USE THEM:**
- ❌ `high-priority` when `priority:high` exists
- ❌ `python` when `task` + description suffices
- ❌ `testing` when `task` + description suffices  
- ❌ `blocker` when `blocked` exists
- ❌ `wip` when `in-progress` exists
- ❌ `p0/p1/p2` when `priority:critical/high/medium` exist
- ❌ Any new label without checking existing ones first

**Why**: Every new label fragments the labeling system. Most needs are met by combining existing labels.

#### Label Hygiene Rules
1. **Check first**: Always run `gh label list` before creating
2. **Reuse existing**: Use exact existing labels, don't create variants
3. **No synonyms**: Don't create `urgent` if `priority:critical` exists
4. **Consistent format**: Follow existing patterns (usually `category:value` or single word)
5. **No personal labels**: Avoid user-specific labels like `john-todo`
6. **Clean up**: If you see duplicate/unused labels, note them for cleanup

#### Safe Label Operations
```bash
# SAFE: Using existing labels
gh issue edit 35 --add-label "priority:high,task"

# UNSAFE: Creating without checking
# gh label create "high-priority"  # DON'T DO THIS

# SAFE: Check what labels an issue has
gh issue view 35 --json labels

# SAFE: Find issues with specific labels
gh issue list --label "priority:high" --label "task"
```

## Project Management

### Creating and Managing Projects
```bash
# Create a new project
gh project create --owner specado --title 'Project Name'

# List projects
gh api graphql -f query='
{
  organization(login: "specado") {
    projectsV2(first: 10) {
      nodes {
        id
        number
        title
        url
      }
    }
  }
}'

# Add issues to project
gh project item-add PROJECT_NUMBER --owner specado --url "https://github.com/specado/specado/issues/ISSUE_NUMBER"
```

### Managing Project Fields
```bash
# Get project fields and their IDs
gh api graphql -f query='
{
  organization(login: "specado") {
    projectV2(number: PROJECT_NUMBER) {
      fields(first: 20) {
        nodes {
          ... on ProjectV2SingleSelectField {
            id
            name
            options {
              id
              name
            }
          }
          ... on ProjectV2Field {
            id
            name
          }
        }
      }
    }
  }
}'

# Create custom fields
gh project field-create PROJECT_NUMBER --owner specado --name 'Field Name' --data-type 'SINGLE_SELECT' --single-select-options 'Option1,Option2'

# Delete a field
gh api graphql -f query='
mutation {
  deleteProjectV2Field(input: {
    fieldId: "FIELD_ID"
  }) {
    clientMutationId
  }
}'
```

### Updating Project Item Fields
```bash
# Get all project items with their IDs
gh api graphql -f query='
{
  organization(login: "specado") {
    projectV2(number: PROJECT_NUMBER) {
      items(first: 50) {
        nodes {
          id
          content {
            ... on Issue {
              number
              title
            }
          }
        }
      }
    }
  }
}'

# Update a single-select field (e.g., Sprint, Status)
gh api graphql -f query='
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "PROJECT_ID"
    itemId: "ITEM_ID"
    fieldId: "FIELD_ID"
    value: { singleSelectOptionId: "OPTION_ID" }
  }) {
    projectV2Item { id }
  }
}'

# Update multiple items efficiently using loops
for item_id in "ITEM_ID1" "ITEM_ID2" "ITEM_ID3"; do
  gh api graphql -f query='
  mutation {
    updateProjectV2ItemFieldValue(input: {
      projectId: "PROJECT_ID"
      itemId: "'$item_id'"
      fieldId: "FIELD_ID"
      value: { singleSelectOptionId: "OPTION_ID" }
    }) {
      projectV2Item { id }
    }
  }'
done
```

## Quick Reference - Common Operations

### Creating Parent-Child Relationships (Most Common Task)
```bash
# Step 1: Get the node IDs
gh api graphql -f query='
{
  repository(owner: "specado", name: "specado") {
    epic: issue(number: 42) { id }
    task1: issue(number: 47) { id }
    task2: issue(number: 48) { id }
  }
}'

# Step 2: Create relationships
gh api graphql -H 'GraphQL-Features: sub_issues' -f query='
mutation {
  rel1: addSubIssue(input: {
    issueId: "EPIC_ID"
    subIssueId: "TASK1_ID"
  }) {
    issue { number }
  }
  rel2: addSubIssue(input: {
    issueId: "EPIC_ID"
    subIssueId: "TASK2_ID"
  }) {
    issue { number }
  }
}'
```

### Project Field Updates (Frequent Operation)
```bash
# Get field and option IDs
gh api graphql -f query='
{
  organization(login: "specado") {
    projectV2(number: 5) {
      field: field(name: "Sprint") {
        ... on ProjectV2SingleSelectField {
          id
          options { id name }
        }
      }
    }
  }
}'

# Update multiple items
gh api graphql -f query='
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "PROJECT_ID"
    itemId: "ITEM_ID"
    fieldId: "FIELD_ID"
    value: { singleSelectOptionId: "OPTION_ID" }
  }) {
    projectV2Item { id }
  }
}'
```

### Cleaning Up Issues
```bash
# Remove task lists from epic
gh issue edit 42 --body 'Clean epic description'

# Delete specific comments
comment_id=$(gh api repos/specado/specado/issues/42/comments --jq '.[0].id')
gh api -X DELETE repos/specado/specado/issues/comments/$comment_id

# Close old issues
gh issue close 8 --comment 'Superseded by #42-#76'
```

### Error Recovery
```bash
# If you accidentally close the wrong issue
gh issue reopen NUMBER
gh issue comment NUMBER --body 'Reopened - closed by mistake'

# If you add wrong labels
gh issue edit NUMBER --remove-label 'wrong-label'

# If parent-child relationship fails
# Check that both issues exist and get fresh node IDs
```

## GitHub CLI Commands Reference

```bash
# Issues
gh issue create --title 'Title' --body 'Body' --label 'label1,label2'
gh issue list --state open --limit 20
gh issue view 35
gh issue close 35 --comment 'Completion message'
gh issue comment 35 --body 'Update message'
gh issue edit 35 --add-label 'in-progress'

# PRs
gh pr create --title 'Title' --body 'Body'
gh pr list --state open
gh pr view 123
gh pr comment 123 --body 'Review feedback'
gh pr merge 123 --squash

# Labels - ALWAYS LIST FIRST
gh label list --limit 100  # Check existing labels FIRST
gh label list --search 'term'  # Search for specific labels
gh label create 'new-label' --description 'Description' --color '0366d6'  # ONLY if truly needed
gh issue view 35 --json labels  # Check issue's current labels

# Milestones
gh issue list --milestone 'Sprint 2'

# Projects
gh project list --owner specado
gh project view PROJECT_NUMBER --owner specado
gh project item-add PROJECT_NUMBER --owner specado --url 'ISSUE_URL'
gh project field-create PROJECT_NUMBER --owner specado --name 'Field' --data-type 'SINGLE_SELECT'
```

## Comprehensive Implementation Workflow

### Phase 1: Planning and Documentation
1. **Create Implementation Plan**
   - Review requirements document (e.g., specado-llm-refactor.md)
   - Create comprehensive plan with epics, tasks, dependencies
   - Include risk assessments and time estimates
   - Document in markdown file for reference

2. **Verify Architecture**
   - Confirm layered architecture approach
   - Document component relationships
   - Create architecture-overview.md if needed

### Phase 2: GitHub Issue Creation
1. **Create Epics First**
   ```bash
   gh issue create --title 'Epic: Core Library Foundation' \
     --body 'Epic description' \
     --label 'epic'
   ```

2. **Create Tasks for Each Epic**
   ```bash
   gh issue create --title 'Task: Implement Provider trait' \
     --body 'Task description with acceptance criteria' \
     --label 'task'
   ```

3. **Establish Parent-Child Relationships**
   ```bash
   # Get node IDs first
   gh api graphql -f query='
   {
     repository(owner: "org", name: "repo") {
       epic: issue(number: 42) { id }
       task: issue(number: 47) { id }
     }
   }'
   
   # Create relationship
   gh api graphql -H 'GraphQL-Features: sub_issues' -f query='
   mutation {
     addSubIssue(input: {
       issueId: "EPIC_ID"
       subIssueId: "TASK_ID"
     }) {
       issue { number }
       subIssue { number }
     }
   }'
   ```

### Phase 3: Project Setup
1. **Create Project**
   ```bash
   gh project create --owner org --title 'Implementation Project'
   ```

2. **Add All Issues**
   ```bash
   # Add issues individually (no scripts)
   gh project item-add 5 --owner org --url 'https://github.com/org/repo/issues/42'
   ```

3. **Configure Custom Fields**
   ```bash
   # Create Sprint field
   gh project field-create 5 --owner org --name 'Sprint' \
     --data-type 'SINGLE_SELECT' \
     --single-select-options 'Sprint 1,Sprint 2,Sprint 3'
   
   # Create Is Epic field
   gh project field-create 5 --owner org --name 'Is Epic' \
     --data-type 'SINGLE_SELECT' \
     --single-select-options 'Yes,No'
   ```

4. **Update Field Values**
   ```bash
   # Use GraphQL to update field values
   gh api graphql -f query='
   mutation {
     updateProjectV2ItemFieldValue(input: {
       projectId: "PROJECT_ID"
       itemId: "ITEM_ID"
       fieldId: "FIELD_ID"
       value: { singleSelectOptionId: "OPTION_ID" }
     }) {
       projectV2Item { id }
     }
   }'
   ```

### Phase 4: Issue Cleanup
1. **Remove Redundant Comments**
   ```bash
   # Check and delete specific comments
   comment_id=$(gh api repos/org/repo/issues/42/comments --jq '.[0].id')
   gh api -X DELETE repos/org/repo/issues/comments/$comment_id
   ```

2. **Clean Epic Descriptions**
   ```bash
   gh issue edit 42 --body 'Clean epic description without task lists'
   ```

3. **Close Superseded Issues**
   ```bash
   gh issue close 8 --comment 'Superseded by new implementation plan in issues #42-#76'
   ```

## Common Pitfalls and Solutions

### Issue Management Pitfalls
1. **Using Task Lists Instead of Parent-Child Relationships**
   - ❌ Wrong: Adding `- [ ] #47` in epic description
   - ✅ Right: Use GraphQL `addSubIssue` mutation for proper tracking

2. **Creating Comments That Create Noise**
   - ❌ Wrong: Adding "Part of Epic #42" comments after relationships exist
   - ✅ Right: Rely on GitHub's native parent-child UI

3. **Not Checking for Existing Issues**
   - ❌ Wrong: Creating new issue without searching
   - ✅ Right: `gh issue list --search 'topic' --state all` first

### Project Management Pitfalls
1. **Trying to Change Field Types (This should be rare)**
   - ❌ Wrong: Attempting to modify existing field from text to single-select
   - ✅ Right: Delete field and recreate with correct type

2. **Using Scripts for Batch Updates**
   - ❌ Wrong: Creating shell scripts for batch operations
   - ✅ Right: Apply changes directly with individual gh commands

3. **Not Getting Correct IDs for GraphQL**
   - ❌ Wrong: Using issue numbers in GraphQL mutations
   - ✅ Right: Query for node IDs first, then use in mutations

### Command Syntax Pitfalls
1. **Using Double Quotes in gh Commands**
   - ❌ Wrong: `gh issue create --title "Title"`
   - ✅ Right: `gh issue create --title 'Title'`

2. **Complex Nested Quotes**
   - ❌ Wrong: Mixing quote types incorrectly
   - ✅ Right: Use single quotes for gh, escape internal quotes as needed

### Cleanup Pitfalls
1. **Leaving Redundant Information**
   - ❌ Wrong: Keeping task lists after creating parent-child relationships
   - ✅ Right: Clean epic descriptions to be focused and concise

2. **Not Closing Superseded Issues**
   - ❌ Wrong: Leaving old issues open when replaced
   - ✅ Right: Close with clear explanation of replacement

## 🚨 Most Common AI Assistant Mistakes

1. **Creating labels without checking**
   - Always run `gh label list` first
   - 90% of the time, the label already exists

2. **Using double quotes in gh commands**
   - Single quotes work reliably
   - Double quotes can cause parsing issues

3. **Not providing evidence when closing**
   - Always include what was implemented
   - Reference commits or explain why closing

4. **Creating issues without searching**
   - Someone may have already created it
   - Check closed issues too - might just need reopening

5. **Forgetting to update parent issues**
   - When completing a sub-task, comment on the epic
   - Update blockers when dependencies resolve

## Lessons Learned

### Effective Practices
1. **Document Everything First**
   - Create comprehensive plan before GitHub operations
   - Include architecture diagrams and component relationships
   - Document decisions for future reference

2. **Use Native GitHub Features**
   - Parent-child relationships > task lists
   - Project fields for organization
   - GraphQL for advanced operations

3. **Maintain Clean Issues**
   - Remove unnecessary comments
   - Keep descriptions focused
   - Update rather than append information

4. **Batch Operations Wisely**
   - Group related operations
   - Use GraphQL for complex updates
   - Apply changes incrementally, not via scripts

### Performance Tips
1. **Query Once, Use Multiple Times**
   - Get all node IDs in one GraphQL query
   - Cache project and field IDs for session

2. **Use Appropriate Tools**
   - GraphQL for relationships and complex updates
   - REST API (via gh) for simple operations
   - Native gh commands when available

3. **Incremental Updates**
   - Update issues as you go
   - Don't wait to batch everything
   - Verify each step before proceeding

## Templates

### Quick Issue Update
```markdown
**Update**: [What changed]
- [Detail 1]
- [Detail 2]
[Next steps or blockers]
```

### Dependency Resolved
```markdown
**Dependency Update**: #[number] has been completed.
This unblocks [what can now be done].
Ready to proceed with implementation.
```

### Work Completed
```markdown
✅ **Completed** in [commit-sha]
- All acceptance criteria met
- Tests passing
- Documentation updated
See [file/path] for implementation.
```

### Crate/Package Renaming
```markdown
**Completed**: Renamed all crates from spec-* to specado-*

Changes included:
- ✅ Directory renames: crates/spec-* → crates/specado-*
- ✅ Cargo.toml updates (workspace and dependencies)
- ✅ Import updates throughout codebase
- ✅ CI/CD workflow path updates

See commit [sha] for implementation.
```