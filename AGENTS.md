# Agents Development Guide

**Purpose**: This document defines the mandatory principles, processes, and practices that **all AI agents** (human or model-based) must follow when working in this project. Strict adherence prevents unpredictable behavior, reduces technical debt, and ensures maintainable, decoupled, high-quality outcomes.

**Audience**: Project owner (Phil) and all AI collaborators only.

**Core Rule (Non-Negotiable)**:  
**Always use Socratic Questioning to clarify any unclear part before doing anything.**  
Ask targeted questions until the full picture is crystal clear. Do not assume or proceed with ambiguity.

**Context Management Rule (Non-Negotiable)**:  
All plans, summaries, decisions, and key context must be tidied up and output (or appended) to `CONTEXT.md`.  
**Always refer to CONTEXT.md first** to ensure no context is missing before starting any task, analysis, or implementation.
**Speak like Linus Torvalds**: direct and simple wordings, never speak out useless sentense.
**Fork sub-agents for works if no dependency**: to speed up the process, just fork agents for all no dependency works.

---

## 1. Core Principles for All Agents

### 1.1 SOLID Principles (Non-Negotiable)
We enforce **S**ingle Responsibility, **O**pen/Closed, **I**nterface Segregation as foundational. These ensure code separation and prevent cascading ("landslide") failures.

- **Single Responsibility Principle (SRP)**: Every class, module, agent, or function has one clear responsibility. If it does more than one thing, split it.
- **Open/Closed Principle (OCP)**: Entities should be open for extension (via composition, strategies, plugins) but closed for modification.
- **Interface Segregation Principle (ISP)**: Prefer many small, client-specific interfaces over one large "god" interface.

**Dependency Inversion** and **Liskov Substitution** are strongly encouraged where they add value.

### 1.2 Additional Clean & Decoupled Practices (Strongly Recommended)
- **Dependency Injection** (explicit, never hidden)
- **Composition over Inheritance**
- **Clean/Hexagonal Architecture** where appropriate (core domain isolated from infrastructure)
- **Loose Coupling + High Cohesion**
- **YAGNI** (You Aren't Gonna Need It) + **KISS** (Keep It Simple, Stupid)
- **DRY** (only when it doesn't harm readability or flexibility)
- **Vertical Slicing** for features when beneficial
- Clear contracts (interfaces/abstract base classes) between components
- Observability (logging, metrics, tracing) built-in from the start
- Security boundaries and input validation enforced early

### 1.3 Predictability Mandate
All AI actions must produce **consistent, explainable, and reproducible** results. No "creative" deviations without explicit approval via Socratic clarification.

---

## 2. Test-Driven Development (TDD) Process

**Mandatory Workflow**: **Red → Green → Refactor**

1. **Red**: Write failing test(s) that clearly define the expected behavior.
2. **Green**: Write the minimal code needed to make the test(s) pass.
3. **Refactor**: Improve the code (and tests) while keeping all tests green. Apply SOLID and clean practices.

**Test Types & Purposes** (use the right test for the right purpose):

- **Unit Tests**: Isolate and verify individual components (functions, classes). Fast, high coverage expected. Mock external dependencies.
- **Integration Tests**: Verify interactions between components (e.g., agent + tool, service + repository).
- **Contract Tests**: Ensure interfaces/contracts between agents/modules are respected.
- **E2E / Behavior Tests**: Validate end-to-end flows from the user's or system's perspective.
- **Property-Based / Edge-Case Tests**: Generated or explicit tests for robustness.

**TDD Rules for AI Agents**:
- Always write tests **before** implementation.
- Tests must be clear, deterministic, and fast.
- Aim for high meaningful coverage (focus on behavior, not just lines).
- Use test fixtures, factories, and proper mocking strategies.
- After every Red-Green-Refactor cycle, run the full relevant test suite.
- Refactor aggressively but safely — never break green tests.
- After test is written, **STOP** current work. Manual review on test is required.
    - When review done without issue, dev task will be resume by prompt.

---

## 3. Working with Different AI Models

Different models have different strengths and failure modes. All models must:

- Follow this `agents.md` strictly.
- Declare which model they are using when relevant.
- Adapt prompting style to their capabilities while never violating principles.
- Use Socratic questioning when model-specific quirks create ambiguity.
- Route complex tasks through the appropriate model only after clarification.

**No model is allowed to bypass TDD, SOLID, or context rules.**

---

## 4. Mandatory Workflow for Any Task

Before starting **any** work:

1. **Read CONTEXT.md** fully.
2. **Apply Socratic Questioning**: Ask clarifying questions (to user or self) until all ambiguities are resolved.
3. **Summarize & Update Context**: Produce a tidy plan/summary and append/update `CONTEXT.md`.
4. **Proceed only when clear**.

**For Implementation Tasks**:
- Follow Red → Green → Refactor strictly.
- Apply SOLID + clean practices at every refactor step.
- Document decisions in `CONTEXT.md`.
- Try build and ensure all tests pass after code change.

**For Analysis / Planning Tasks**:
- Use Socratic method internally.
- Output clear, structured summary to `CONTEXT.md`.

---

## 5. Documentation & Communication Standards

- All major decisions, plans, and summaries go into `CONTEXT.md`.
- Code must be self-documenting where possible (clear names, small functions).
- Use Mermaid diagrams when helpful for architecture or flows.
- Keep this `agents.md` as the single source of truth for agent behavior. Suggest improvements via pull request / discussion only after Socratic validation.

---

## 6. Enforcement & Continuous Improvement

- Every AI response or code contribution must implicitly or explicitly align with this guide.
- Violations will be called out immediately.
- This document is living. Improvements are welcome but must follow the same Socratic + context-updating process.

**Final Reminder**:
> Always use Socratic Questioning to clarify unclear parts before doing anything.  
> All plans and summaries must be tidied up and output to CONTEXT.md. Always refer to it to ensure no context is missing before doing anything.

By following these rules, we create a stable, predictable, and high-quality development environment regardless of which AI model is active.