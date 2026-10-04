<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Demos

Demos are complete, runnable StateLink scenarios involving multiple components.

For small API examples, use [`../examples/README.md`](../examples/README.md) instead.

## Basic producer/consumer demo

Location:

```text
demos/basic/
```

What it demonstrates:

- StateLink reference server;
- authenticated producer and consumer identities;
- system policy;
- consumer subscription before producer startup;
- producer context declaration;
- current-state updates and revisions;
- Docker networking/configuration.

Run from the repository root:

```bash
make demo
```

Windows PowerShell:

```powershell
.\scripts\demo.ps1
```

Or directly:

```bash
docker compose -f demos/basic/docker-compose.yml up --build --abort-on-container-exit
```

Read [`basic/README.md`](basic/README.md) for the flow and expected behavior.

## Demo conventions

Future end-to-end demos should be added as:

```text
demos/<name>/
├── README.md
├── docker-compose.yml   # when Compose is appropriate
└── ...scenario-specific files...
```

Each demo README should include:

- purpose;
- prerequisites;
- exact startup command;
- components/services;
- expected behavior;
- cleanup instructions;
- limitations/security notes.
