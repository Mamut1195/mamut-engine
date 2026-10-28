# CLA process for external contributors

This document relates to ENG-GOV-007 and describes the process for managing the contributor agreement (CLA). It does not contain the contractual text or replace its drafting or legal review.

## Why it exists

The CLA is part of MAMUT Engine's dual licensing model: AGPL-3.0 and a commercial license. Its purpose is to manage the necessary permissions relating to external contributions to sustain that model. The content and legal effects of the agreement remain pending drafting and review by a lawyer.

## Scope

- **Individual:** contributions from an external person contributing in an individual capacity.
- **Corporate:** contributions made on behalf of a company or other entity. How to establish authority to represent the entity and link authors to the corporate signature is **PENDING** definition and legal review.

The identification of external authors and the details of how each form applies are **PENDING** definition.

## Pre-merge rule

**There must be a signature on record before an external author's first merge.** This rule applies to both the individual and corporate scope; in the latter case, it must be verified that the signature on record covers the author.

Implementation of the control and its automated verification are **PENDING**. The documented rule does not imply that the control is already active.

## Planned automated signing workflow in a PR

1. When a PR is opened, the automated workflow identifies external authors and checks whether they have a signature on record that covers them.
2. If a signature is missing, it requests that signing be completed under the applicable individual or corporate form and refers to the contractual text, which will reside only in Notion.
3. Once signing is complete, the workflow records its outcome and checks that it covers the author.
4. Before each external author's first merge, the signature on record is verified. If it is missing, the PR must not be merged.

This workflow is a description of the planned process; putting it into operation is **PENDING**.

- **PR automation tool:** **PENDING** selection and configuration.
- **Signature recording location:** **PENDING** definition. No recording system or location has been established.
- **Verification and merge-blocking mechanism:** **PENDING** implementation.

## Status of the contractual text and ownership

- **CLA text:** **PENDING** drafting and review by a lawyer.
- **Location of the contractual text:** only in **Notion**, outside the repository. The specific page or link is **PENDING** definition.
- **Copyright holder:** **MAMPRO SAS** (confirmed by Jonathan on 2026-10-15; see ADR-001).

The location of the contractual text does not determine where signatures will be recorded; that record remains pending.

## Acceptance criteria

Checked boxes correspond only to documented aspects, not to an operational implementation or legal approval.

- [x] **DOCUMENTED:** explain the purpose of the CLA in the dual AGPL-3.0 and commercial model.
- [x] **DOCUMENTED:** describe the individual and corporate scope.
- [x] **DOCUMENTED:** establish the rule requiring a signature on record before an external author's first merge.
- [x] **DOCUMENTED:** describe the planned automated signing workflow in PRs and indicate that the tool is pending.
- [x] **DOCUMENTED:** indicate that the contractual text will reside only in Notion and not in the repository.
- [x] **DOCUMENTED:** state that drafting and legal review and the signature record are pending (the copyright holder is defined; see ADR-001).
- [ ] **PENDING:** define how external authors are identified and how each form applies.
- [ ] **PENDING:** define and review the establishment of authority to represent the entity and coverage of authors under the corporate form.
- [ ] **PENDING:** draft the contractual text and complete its review by a lawyer.
- [ ] **PENDING:** make the contractual text available in Notion and determine its link.
- [x] **DONE:** determine the copyright holder: MAMPRO SAS (confirmed 2026-10-15; see ADR-001).
- [ ] **PENDING:** select and configure the automated signing tool in PRs.
- [ ] **PENDING:** define and enable the signature recording location.
- [ ] **PENDING:** implement and verify the control that prevents an external author's first merge without a signature on record.
