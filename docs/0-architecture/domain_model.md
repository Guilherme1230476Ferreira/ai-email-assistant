# Backend Domain Model

This document outlines the core entities and their relationships within the Mailwise AI Assistant backend.

## Core Architectural Components

### 1. Identity & Access (IAM)
- **User**: The primary actor. Authenticates via JWT.
- **Role**: Defines permissions (e.g., `admin`, `user`). Currently implemented via a hard-coded RBAC middleware checking the role name.

### 2. Messaging & Intelligence
- **Email**: Represents an incoming email and its AI-generated counterpart.
- **EmailEmbedding**: Stores vector representations of email content to enable **RAG (Retrieval-Augmented Generation)**. Uses `pgvector` for similarity searching.

### 3. Governance & Configuration
- **AppSetting**: Global configuration for LLM providers. Sensitive data (API Keys) are stored encrypted.
- **AuditLog**: Immutable record of administrative actions for security compliance.

### 4. Security Services
- **CryptoService**: Handles AES-256-GCM encryption for secrets at rest.
- **RateLimiter**: Protects authentication endpoints from brute-force attacks using an in-memory sliding window.
