---
id: sec-js/graphql-hardening
title: "A GraphQL server limits query depth and cost, and hides introspection"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the limits that fit the schema are a decision for the author"
stability: preview
security: { cwe: [200, 770], owasp: ["A02:2025"], severity: medium }
since: 0.3.0
---

An Apollo, Yoga or express-graphql server names a depth or cost limit in its options, and does not set `introspection: true`.

**Why:** A query can nest or fan out without bound and exhaust the server, and introspection hands an attacker the whole schema (CWE-770, CWE-200).

```codelaw
match:
  any:
    - all:
        - any:
            - pattern: new ApolloServer($$$A)
            - pattern: createYoga($$$A)
            - pattern: graphqlHTTP($$$A)
            - pattern: createHandler($$$A)
        - not:
            has:
              stopBy: end
              regex: '(?i)depthLimit|costLimit|queryComplexity|maxDepth|validationRules|cost|complexity|armor'
    - pattern:
        context: '({ introspection: true })'
        selector: pair
message: "This GraphQL server has no depth or cost limit, or leaves introspection on"
```

## How to fix

1. Add a depth limit (`graphql-depth-limit`) and a cost or complexity limit to `validationRules`, and cap the page size of list fields.
2. Turn introspection off in production, and use persisted queries where the clients are your own.

## Examples

### JavaScript

#### Don't

```js
const server = new ApolloServer({ typeDefs, resolvers });
```

#### Do

```js
const server = new ApolloServer({
  typeDefs,
  resolvers,
  validationRules: [depthLimit(8)],
  introspection: process.env.NODE_ENV !== "production",
});
```

### JSX

#### Don't

```jsx
const server = new ApolloServer({ typeDefs, resolvers });
```

#### Do

```jsx
const server = new ApolloServer({
  typeDefs,
  resolvers,
  validationRules: [depthLimit(8)],
  introspection: process.env.NODE_ENV !== "production",
});
```

### TypeScript

#### Don't

```ts
const server = new ApolloServer({ typeDefs, resolvers });
```

#### Do

```ts
const server = new ApolloServer({
  typeDefs,
  resolvers,
  validationRules: [depthLimit(8)],
  introspection: process.env.NODE_ENV !== "production",
});
```

### TSX

#### Don't

```tsx
const server = new ApolloServer({ typeDefs, resolvers });
```

#### Do

```tsx
const server = new ApolloServer({
  typeDefs,
  resolvers,
  validationRules: [depthLimit(8)],
  introspection: process.env.NODE_ENV !== "production",
});
```
