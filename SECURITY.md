# Security Policy

Copyright (C) 2026 Gokul Kartha  
SPDX-License-Identifier: GPL-3.0-or-later

StateLink is security-sensitive infrastructure. Please do not open a public issue for a vulnerability that could expose credentials, bypass topic authorization, spoof context ownership, or remotely exhaust a deployed StateLink service.

For now, report security issues privately to the repository owner through GitHub's private vulnerability reporting feature when available.

## Security assumptions

The reference implementation assumes:

- production deployments use TLS (`wss://`) or a trusted local transport;
- client identity is established outside producer-controlled JSON;
- system policy is deny-by-default;
- producer exposure rules can only reduce access;
- browser Origin is an additional control and is never treated as identity;
- static tokens in `config/dev-auth.json` are examples only and must not be deployed.

The initial MQTT adapter implements the data-interoperability profile. A generic MQTT broker does not automatically enforce StateLink ownership or producer exposure policy; broker-side MQTT ACLs and secure transport remain required.
