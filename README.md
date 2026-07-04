# Real-Time Chat Engine (Full-Stack Workspace)

A production-ready, monorepo chat platform featuring a high-performance **Rust & Axum** backend API and a responsive **React (TypeScript)** frontend application. Built to run securely and smoothly using localized containerization.

## Tech Stack & Core Design

### Backend Core (Rust)
* **Framework:** Axum (Tokio runtime) for asynchronous routing.
* **Database & Driver:** PostgreSQL powered by SQLx with compile-time verified queries.
* **Time & Sorting:** **UUIDv7** for instant sequential message IDs and timezone-aware sorting using the `time` crate.
* **State:** Hard deletes for database efficiency and data control.

### Frontend App (React & TypeScript)
* **Framework:** React 18+ with TypeScript for a fully typed design architecture.
* **Build Tooling:** Vite for near-instant hot module replacement (HMR).
* **State & Querying:** Client-side cache tracking with cursor-based infinite scrolling pagination (`limit` & `before` query selectors).

---

## API Routing Map

All protected paths validate JWT session credentials passed dynamically inside request `Claims`.

### Core App Base
* `GET /` — Service health checking and semantic package versioning.

### Authentication Module (`/auth`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/auth/login` | Authenticate client and return token pair. |
| `POST` | `/auth/refresh` | Issue fresh short-lived access tokens via refresh token. |
| `POST` | `/auth/logout` | Revoke ongoing session validity. |

### Profile & Identity Management (`/user`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/user/register` | Provision a new system account. |
| `PATCH` | `/user/update` | Modify user profile attributes. |
| `DELETE` | `/user/delete` | Immediate hard deletion of user account data. |
| `GET` | `/user/me` | Fetch active user's internal metadata details. |
| `GET` | `/user/search` | Dynamic lookups for finding chat contacts. |
| `GET` | `/user/status` | Retrieve custom active states (e.g., Online/Away). |

### Chat Channels Module (`/rooms`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/rooms/` | Instantiate a group room or static DM thread. |
| `GET` | `/rooms/` | Retrieve contextual overview of rooms the caller resides in. |
| `PATCH` | `/rooms/update/{room_id}` | Synchronize group member lists (Sync/Diff matrix) or rename rooms. |
| `DELETE` | `/rooms/delete/{room_id}` | Hard drop a room (restricted to group creators only). |
| `GET` | `/rooms/me/{room_id}` | Fetch room details along with its member list. |

### Messaging Sub-Router (`/rooms/{room_id}/messages`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/rooms/{room_id}/messages/` | Dispatch a message. Membership evaluated within an atomic transaction block. |
| `GET` | `/rooms/{room_id}/messages/` | Load history using query parameters (`?limit=XX&before=TIMESTAMPTZ`). |

---

## Containerization & Local Orchestration (Docker)

The workspace is split into lightweight, production-optimized container images using multi-stage pipelines to minimize deployment footprints.

### Multi-Stage Architectures
* **Backend Target:** Built using `cargo-chef` to isolate and cache dependencies. The final production asset is packaged inside a ~20MB bare `debian-slim` runtime execution container.
* **Frontend Target:** Compiled via Node/Vite build stages into plain static files, instantly served through an optimized, low-footprint `nginx:alpine` routing container.

### Local Initialization
To launch the entire development environment—including the database engine, the backend API runner, and the web app workspace—execute the orchestrator from the project root:

```bash
docker compose up --build
```

## Environmental Settings

Create a `.env` configuration template in your project workspace directory root:

```env
DATABASE_URL="postgres://username:password@localhost:5432/chat_db"
JWT_SECRET="your-ultra-secure-cryptographic-signing-key"
HOST="127.0.0.1"
PORT="8080"
