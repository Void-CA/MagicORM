# MagicORM

## Why does MagicORM exist?

MagicORM was born from real-world problems.

Not from a theoretical comparison with other ORMs, but from the experience of developing systems that constantly change.

During rapid iteration in evolving systems, the same obstacles always appeared:

- Too much boilerplate.
- Repetitive adjustments when requirements changed.
- Costly refactors for small structural changes.
- Data access code growing faster than the domain itself.

In the early stages of a project, when everything is evolving, friction doesn't come from complex queries.
It comes from repetition.

### The problem: iterating quickly without infrastructure slowing you down

When a system is still being defined:

- Models change.
- Relationships change.
- Fields change.
- Rules change.

And every small change ends up impacting:

- Mappings.
- Builders.
- Configurations.
- Repetitive implementations.
- Code that doesn't add real value to the domain.

Infrastructure starts to dominate the code.

MagicORM arises as a direct response to this.

## The Goal

Reduce friction in fast-evolving systems.

MagicORM is designed to:

- Minimize boilerplate.
- Reduce the amount of code affected by change.
- Centralize conventions.
- Automate repetitive patterns.
- Allow the domain model to be the main focus.

## Design Philosophy

### 1. Iteration First

MagicORM prioritizes the ability to change the system without every modification requiring a rewrite of half the persistence layer.

If the domain evolves, the data layer should adapt with minimal friction.

### 2. Aggressive Abstraction

MagicORM doesn't try to expose every detail.
It tries to hide them.

Repetitive decisions become conventions.
Standard configurations become implicit behavior.

Yes, this means less structural freedom.
But in exchange:

- Less repeated code.
- Fewer points of failure.
- Less surface to maintain.

### 3. Conventions Over Configuration

Many maintenance problems arise from excessive flexibility.

MagicORM adopts strong conventions to avoid:

- Redundant configuration.
- Inconsistent structures.
- Different styles within the same project.

Consistency is not accidental. It is enforced.

### 4. Boilerplate as a Symptom

If to perform a common operation you need to:

- Define auxiliary structures.
- Implement multiple traits.
- Configure extensive builders.
- Repeat patterns over and over.

Then the system is leaking infrastructure into the domain.

MagicORM seeks to reduce this to a minimum.

## The Conscious Trade-off

MagicORM is not a tool for those who need absolute control over every detail.

The trade-off is clear:

| What is reduced                | What is gained         |
|-------------------------------|------------------------|
| Total structural flexibility   | Iteration speed        |
| Exhaustive configuration       | Simplicity             |
| Manual granular control        | Abstraction            |
| Explicit boilerplate           | Fluidity               |

It is not designed for extreme optimization.
It is designed for evolutionary development.

## What MagicORM Tries to Be

- A facilitator during early growth stages.
- A layer that absorbs frequent changes.
- A tool that encourages experimentation and refactoring.
- A system that reduces cognitive load when modifying the model.

## Current State

MagicORM is in the design phase.

The syntax is not yet defined.
The current priority is to establish solid principles before a definitive API.

First, solve the friction.
Then, design the tool.