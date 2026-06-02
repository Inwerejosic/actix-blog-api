Below is a **detailed README** for the project, plus a **GitHub Actions workflow** that builds the Docker image and pushes it to both **GitHub Container Registry (GHCR)** and **Docker Hub**.

---

# 📝 Blog API – Actix‑web + PostgreSQL + Redis

A production‑ready REST API for a blog platform, built with **Rust**, **Actix‑web**, **PostgreSQL**, **Redis** (for session management), and **SQLx**. Supports user authentication (sessions), blog posts, tags, and many‑to‑many relationships.

---

## 🚀 Features

- User registration & login (session‑based, Redis backend)
- Protected endpoints (e.g., `/me`)
- CRUD operations for users, blog posts, and tags
- Many‑to‑many association between posts and tags
- PostgreSQL for persistent storage, Redis for session caching
- Dockerised with a **minimal final image** (~15 MB)
- GitHub Actions CI/CD – pushes to GHCR and Docker Hub

---

## 📦 Tech Stack

| Component       | Technology                         |
|----------------|------------------------------------|
| Web framework   | Actix‑web 4                        |
| Database        | PostgreSQL 16 (with SQLx)          |
| Session store   | Redis 7 (via `actix-session`)      |
| Authentication  | bcrypt + session cookies           |
| Containerisation| Docker + Docker Compose            |
| CI/CD           | GitHub Actions                     |

---

## 🛠️ Local Development (without Docker)

### Prerequisites

- Rust 1.88 or later  
- PostgreSQL 16  
- Redis 7  
- `.env` file with:

```env
DATABASE_URL=postgres://postgres:password@localhost/blog_db
REDIS_URL=redis://127.0.0.1:6379
SECRET_KEY=<64‑byte hex key – generate with `openssl rand -hex 64`>
```

### Run

```bash
cargo run
```

Server starts at `http://127.0.0.1:8080`.

---

## 🐳 Run with Docker Compose (recommended)

No local Rust/PostgreSQL/Redis needed – everything runs in containers.

```bash
docker compose up --build
```

The API is available at `http://localhost:8080`.

To stop:

```bash
docker compose down
```

Data persists in Docker volumes (`postgres_data`, `redis_data`).

---

## 📚 API Endpoints

Base URL: `http://localhost:8080`

### 🔐 Authentication

| Method | Endpoint       | Description                     |
|--------|----------------|---------------------------------|
| POST   | `/login`       | Login (stores session cookie)   |
| POST   | `/logout`      | Logout (destroys session)       |
| GET    | `/me`          | Get current authenticated user  |

### 👤 Users

| Method | Endpoint        | Description                |
|--------|-----------------|----------------------------|
| GET    | `/users`        | List all users             |
| GET    | `/users/{id}`   | Get user by ID             |
| POST   | `/users`        | Create a new user          |
| PUT    | `/users/{id}`   | Update user                |
| DELETE | `/users/{id}`   | Delete user                |

### 📄 Blog Posts

| Method | Endpoint          | Description                       |
|--------|-------------------|-----------------------------------|
| GET    | `/posts`          | List all posts                    |
| POST   | `/posts`          | Create a new post                 |
| GET    | `/posts/{id}`     | Get a single post with its tags   |

### 🏷️ Tags & Relations

| Method   | Endpoint                        | Description                |
|----------|---------------------------------|----------------------------|
| GET      | `/tags`                         | List all tags              |
| POST     | `/tags`                         | Create a tag               |
| POST     | `/posts/{post_id}/tags/{tag_id}`| Attach tag to post         |
| DELETE   | `/posts/{post_id}/tags/{tag_id}`| Detach tag from post       |

> **Note:** Comments are defined in the database schema but are not implemented in handlers – ready for extension.

---

## 🧪 Example `curl` Commands

```bash
# 1. Register a user
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret","email":"alice@example.com"}'

# 2. Login (store cookie)
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret"}' \
  -c cookies.txt

# 3. Get current user (authenticated)
curl http://localhost:8080/me -b cookies.txt

# 4. Create a post
curl -X POST http://localhost:8080/posts \
  -H "Content-Type: application/json" \
  -d '{"user_id":1,"title":"Hello","content":"World","cover":null}' \
  -b cookies.txt

# 5. Create a tag and attach it to the post
curl -X POST http://localhost:8080/tags -H "Content-Type: application/json" -d '{"name":"rust"}'
curl -X POST http://localhost:8080/posts/1/tags/1

# 6. Get post with tags
curl http://localhost:8080/posts/1

# 7. Logout
curl -X POST http://localhost:8080/logout -b cookies.txt
```

---

## 🗄️ Database Schema

Tables created automatically on first run:

- `users` – id, username, password (bcrypt), email, last_login  
- `blog_posts` – id, user_id (FK), title, content, cover  
- `comments` – id, blog_id (FK), user_id (FK), content  
- `tags` – id, name  
- `blog_tag` – junction table (blog_id, tag_id)

---

## 🔧 Configuration

Environment variables (set in `.env` or Docker Compose):

| Variable       | Description                                         |
|----------------|-----------------------------------------------------|
| `DATABASE_URL` | PostgreSQL connection string                        |
| `REDIS_URL`    | Redis connection string                             |
| `SECRET_KEY`   | 128‑character hex string (64 bytes) – **required**  |

Generate a secure key:

```bash
openssl rand -hex 64
```

---

## 🐙 GitHub Actions – Build & Push to GHCR / Docker Hub

The workflow below will:
- Build the Docker image (using the existing `Dockerfile`)
- Tag it with `latest` and the Git SHA
- Push to both **GitHub Container Registry** (`ghcr.io`) and **Docker Hub**


### 🔑 Required GitHub Secrets

- `DOCKER_USERNAME` – your Docker Hub username  
- `DOCKER_PASSWORD` – your Docker Hub password (or token)

No secret is needed for GHCR – it uses the built‑in `GITHUB_TOKEN`.

---

## 📄 License

MIT – use freely.

---

## 🙌 Acknowledgements

Built with Actix, SQLx, and the Rust ecosystem.

---

**Happy blogging!** 🦀