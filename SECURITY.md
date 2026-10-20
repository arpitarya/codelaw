# Security policy

## Reporting a vulnerability

**Please don't open a public issue.** Report it privately through GitHub instead:

1. Go to the repository's **Security** tab.
2. Choose **Report a vulnerability**.

You'll get an acknowledgement within 5 working days, and a fix or a plan within 30 days of confirming the issue.

## What counts

codelaw reads source files and writes fixes back to them, so these are in scope:

- a fix that writes **outside the files being checked**, or follows a symlink out of the project;
- a crafted source, rule or config file that makes codelaw **run code**, read files it shouldn't, or reach the network;
- a crafted rule pack whose fixes **corrupt files** that the fix verifier should have protected;
- anything in a published package that differs from what's in this repository.

A wrong or noisy finding isn't a security issue. Please [open a false-positive issue](https://github.com/arpitarya/codelaw/issues/new?template=false_positive.yml) for those.

## Supported versions

codelaw is a preview (0.1.x). Fixes go into the latest version only.
