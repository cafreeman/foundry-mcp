# Cursor Custom Command Examples

Docs can be found [here](https://cursor.com/docs/agent/chat/commands).

---

# Code Review Checklist

## Overview

Comprehensive checklist for conducting thorough code reviews to ensure quality, security, and maintainability.

## Review Categories

### Functionality

- [ ] Code does what it's supposed to do
- [ ] Edge cases are handled
- [ ] Error handling is appropriate
- [ ] No obvious bugs or logic errors

### Code Quality

- [ ] Code is readable and well-structured
- [ ] Functions are small and focused
- [ ] Variable names are descriptive
- [ ] No code duplication
- [ ] Follows project conventions

### Security

- [ ] No obvious security vulnerabilities
- [ ] Input validation is present
- [ ] Sensitive data is handled properly
- [ ] No hardcoded secrets

---

# Security Audit

## Overview

Comprehensive security review to identify and fix vulnerabilities in the codebase.

## Steps

1. **Dependency audit**

   - Check for known vulnerabilities
   - Update outdated packages
   - Review third-party dependencies

2. **Code security review**

   - Check for common vulnerabilities
   - Review authentication/authorization
   - Audit data handling practices

3. **Infrastructure security**
   - Review environment variables
   - Check access controls
   - Audit network security

## Security Checklist

- [ ] Dependencies updated and secure
- [ ] No hardcoded secrets
- [ ] Input validation implemented
- [ ] Authentication secure
- [ ] Authorization properly configured

---

# Setup New Feature

## Overview

Systematically set up a new feature from initial planning through to implementation structure.

## Steps

1. **Define requirements**

   - Clarify feature scope and goals
   - Identify user stories and acceptance criteria
   - Plan technical approach

2. **Create feature branch**

   - Branch from main/develop
   - Set up local development environment
   - Configure any new dependencies

3. **Plan architecture**
   - Design data models and APIs
   - Plan UI components and flow
   - Consider testing strategy

## Feature Setup Checklist

- [ ] Requirements documented
- [ ] User stories written
- [ ] Technical approach planned
- [ ] Feature branch created
- [ ] Development environment ready

---

# Create PR

## Overview

Create a well-structured pull request with proper description, labels, and reviewers.

## Steps

1. **Prepare branch**

   - Ensure all changes are committed
   - Push branch to remote
   - Verify branch is up to date with main

2. **Write PR description**

   - Summarize changes clearly
   - Include context and motivation
   - List any breaking changes
   - Add screenshots if UI changes

3. **Set up PR**
   - Create PR with descriptive title
   - Add appropriate labels
   - Assign reviewers
   - Link related issues

## PR Template

- [ ] Feature A
- [ ] Bug fix B
- [ ] Unit tests pass
- [ ] Manual testing completed

---

# Run All Tests and Fix Failures

## Overview

Execute the full test suite and systematically fix any failures, ensuring code quality and functionality.

## Steps

1. **Run test suite**

   - Execute all tests in the project
   - Capture output and identify failures
   - Check both unit and integration tests

2. **Analyze failures**

   - Categorize by type: flaky, broken, new failures
   - Prioritize fixes based on impact
   - Check if failures are related to recent changes

3. **Fix issues systematically**
   - Start with the most critical failures
   - Fix one issue at a time
   - Re-run tests after each fix
