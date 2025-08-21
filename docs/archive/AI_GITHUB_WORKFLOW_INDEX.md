# AI GitHub Workflow Documentation Index

## 📚 Document Overview
**Purpose**: Comprehensive guidelines for AI assistants working with GitHub operations in the Specado project  
**Version**: 1.0  
**Last Updated**: 2025-01-31  
**Primary Audience**: AI assistants and developers implementing GitHub workflows  
**Document Type**: Reference guide and operational handbook  

## 📑 Table of Contents

### [🚀 Quick Start](#quick-start)
- [Pre-Operation Checklist](#quick-start-checklist)
- [Critical Rules](#critical-rules)
- [Common Operations Reference](#quick-reference)

### [📋 Issue Management](#issue-management)
- [Issue Creation](#creating-issues)
- [Issue Templates](#issue-templates)
- [Issue Closure](#closing-issues)
- [Progress Tracking](#progress-comments)
- [Cross-References](#issue-linking)

### [🔀 Pull Request Management](#pull-request-management)
- [PR Descriptions](#pr-descriptions)
- [PR Comments](#pr-comments)
- [Review Process](#pr-reviews)

### [💬 Commit Standards](#commit-standards)
- [Message Format](#commit-format)
- [Type Conventions](#commit-types)
- [Examples](#commit-examples)

### [🏷️ Label Management](#label-management)
- [Label Guidelines](#label-guidelines)
- [Label Categories](#label-categories)
- [Anti-Patterns](#label-anti-patterns)
- [Hygiene Rules](#label-hygiene)

### [📊 Epic & Milestone Management](#epic-management)
- [Epic Structure](#epic-structure)
- [Parent-Child Relationships](#parent-child-relationships)
- [GraphQL Operations](#graphql-operations)

### [🗂️ Project Management](#project-management)
- [Project Creation](#creating-projects)
- [Field Management](#managing-fields)
- [Item Updates](#updating-items)

### [🔧 GitHub CLI Reference](#cli-reference)
- [Issues Commands](#issues-commands)
- [PR Commands](#pr-commands)
- [Label Commands](#label-commands)
- [Project Commands](#project-commands)

### [⚠️ Common Pitfalls](#common-pitfalls)
- [Issue Management Mistakes](#issue-pitfalls)
- [Project Management Errors](#project-pitfalls)
- [Command Syntax Issues](#syntax-pitfalls)

### [📝 Templates](#templates-section)
- [Issue Templates](#issue-update-templates)
- [PR Templates](#pr-templates)
- [Comment Templates](#comment-templates)

## 🎯 Key Concepts

### Core Principles
1. **Check Before Creating**: Always verify existence before creating issues, labels, or projects
2. **Evidence-Based Closure**: Provide implementation details when closing issues
3. **Relationship Management**: Use native GitHub parent-child relationships over task lists
4. **Clean Documentation**: Maintain focused, actionable issue descriptions
5. **Single Quote Convention**: Use single quotes in all `gh` CLI commands

### Workflow Hierarchy
```
Epic (Strategic Goal)
  └── Task (Implementation Unit)
      └── Sub-task (Optional breakdown)
          └── Comments (Progress updates)
```

### Label Philosophy
- **Reuse Over Creation**: Use existing labels whenever possible
- **Consistent Patterns**: Follow `category:value` or single-word format
- **No Variants**: Never create synonyms of existing labels
- **Minimal Set**: Most needs met by combining existing labels

## 🔍 Quick Navigation

### By Task Type

#### **Creating New Work**
- [Issue Creation](#creating-issues) → [Issue Templates](#issue-templates) → [Label Assignment](#label-guidelines)

#### **Tracking Progress**
- [Progress Comments](#progress-comments) → [Status Labels](#label-categories) → [Project Updates](#updating-items)

#### **Completing Work**
- [Issue Closure](#closing-issues) → [PR Creation](#pr-descriptions) → [Commit Standards](#commit-standards)

#### **Organizing Work**
- [Epic Creation](#epic-structure) → [Parent-Child Setup](#parent-child-relationships) → [Project Management](#project-management)

### By GitHub Feature

#### **Issues**
- [Creation](#creating-issues)
- [Management](#issue-management)
- [Closure](#closing-issues)
- [Linking](#issue-linking)

#### **Pull Requests**
- [Creation](#pr-descriptions)
- [Reviews](#pr-comments)
- [Merging](#pr-management)

#### **Projects**
- [Setup](#creating-projects)
- [Fields](#managing-fields)
- [Updates](#updating-items)

#### **Labels**
- [Guidelines](#label-guidelines)
- [Categories](#label-categories)
- [Management](#label-management)

## 📊 Command Quick Reference

### Most Used Commands
```bash
# Check before creating
gh issue list --search "topic" --state all
gh label list --limit 100

# Issue operations
gh issue create --title 'Title' --body 'Body' --label 'label1,label2'
gh issue close 35 --comment 'Completion message'
gh issue edit 35 --add-label 'in-progress'

# Parent-child relationships (GraphQL)
gh api graphql -H 'GraphQL-Features: sub_issues' -f query='mutation...'

# Project updates
gh project item-add PROJECT_NUMBER --owner specado --url 'ISSUE_URL'
```

## 🚨 Critical Anti-Patterns

### Never Do These
1. ❌ Create issues without searching first
2. ❌ Create label variants (e.g., `high-priority` when `priority:high` exists)
3. ❌ Use double quotes in `gh` commands
4. ❌ Close issues without explanation
5. ❌ Use task lists instead of parent-child relationships
6. ❌ Create scripts for batch operations
7. ❌ Leave redundant comments after establishing relationships

### Always Do These
1. ✅ Search existing issues and labels first
2. ✅ Use exact existing labels
3. ✅ Use single quotes in `gh` commands
4. ✅ Provide evidence when closing issues
5. ✅ Use GraphQL for parent-child relationships
6. ✅ Apply changes directly with individual commands
7. ✅ Clean up redundant information

## 📈 Workflow Diagrams

### Issue Lifecycle
```
Create → Label → Assign → In Progress → Review → Close
   ↓        ↓        ↓          ↓           ↓        ↓
Search   Check    Update    Comment    Validate  Evidence
```

### Parent-Child Relationship Flow
```
1. Create Epic Issue
2. Create Task Issues
3. Get Node IDs (GraphQL)
4. Establish Relationships (addSubIssue)
5. Clean Epic Description
6. Add to Project
```

### Label Decision Tree
```
Need Label?
    ├── Yes → Check Existing (`gh label list`)
    │           ├── Found → Use Existing
    │           └── Not Found → Really Needed?
    │                            ├── Yes → Create with Pattern
    │                            └── No → Use Combination
    └── No → Continue
```

## 🔗 Related Documentation

### Internal References
- Implementation plans and architecture documents
- Sprint planning and project documentation
- CI/CD workflow configurations

### External References
- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [GitHub GraphQL API](https://docs.github.com/en/graphql)
- [GitHub Projects Documentation](https://docs.github.com/en/issues/planning-and-tracking-with-projects)

## 📝 Document Metadata

### Maintenance
- **Review Frequency**: Monthly or as GitHub features change
- **Update Authority**: Project maintainers and AI assistants
- **Version Control**: Track changes in git history

### Compliance
- **Standards**: GitHub best practices and Specado conventions
- **Validation**: Peer review and practical application testing
- **Feedback**: Continuous improvement through usage experience

## 🎓 Learning Path

### For New AI Assistants
1. Read [Quick Start](#quick-start) and [Critical Rules](#critical-rules)
2. Review [Common Pitfalls](#common-pitfalls)
3. Practice with [Templates](#templates-section)
4. Reference [CLI Commands](#cli-reference) as needed

### For Advanced Operations
1. Master [GraphQL Operations](#graphql-operations)
2. Understand [Project Management](#project-management)
3. Implement [Workflow Automation](#workflow-automation)
4. Optimize with [Performance Tips](#performance-tips)

---

*This index provides comprehensive navigation and reference for the AI GitHub Workflow Guidelines. Use the quick navigation sections to find specific information efficiently.*