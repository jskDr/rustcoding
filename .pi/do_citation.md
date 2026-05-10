# Skill: Adding Document Citations

## Context
Apply this skill whenever you incorporate information, guidelines, patterns, or summaries from an existing file into another document. This ensures traceability and allows users to find the source of truth.

## Standards & Constraints
- **Format:** Use the `[@filename]` syntax for inline citations.
- **Accuracy:** The filename must exactly match the relative path of the file being cited.
- **Placement:** 
    - Place inline citations immediately after the paragraph or sentence containing the cited information.
    - If the document has a `## References` or `## Citations` section, add a corresponding entry there.
- **Prohibited:** Do not use vague references like "as mentioned in the documentation" or "see other files." Always specify the exact file.

## Workflow
1. **Identify the Source:** Determine which file(s) provided the information you are using.
2. **Verify Existence:** Ensure the target file exists in the current working directory (using `ls` or `find` if unsure).
3. **Insert Inline Citation:** Append `[@filename]` to the relevant text.
4. **Update Reference Section:** 
    - Check if the target document already has a `## References` section.
    - If yes, add the file to the list.
    - If no, and the amount of citations is significant (3+), create a `## References` section at the end of the document.

## Code Examples

### ❌ Incorrect
"The project follows the standard prompt engineering guidelines." 
*(Vague, no source provided)*

"The project follows the standard prompt engineering guidelines (see skill_how_to.md)."
*(Wrong format)*

### ✅ Correct
"The project follows the standard prompt engineering guidelines [@skill_how_to.md]."

**And in the reference section:**
```markdown
## References
- [@skill_how_to.md]: Guide on writing skill markdown files.
```

## Success Criteria
- [ ] Every piece of external information is attributed to a specific file.
- [ ] All citations use the `[@filename]` format.
- [ ] The cited files actually exist in the repository.
- [ ] The Reference section (if applicable) is up-to-date.
