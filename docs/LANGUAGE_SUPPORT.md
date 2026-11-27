# BRAHM Stage 02 Language, Framework & ORM Coverage

## 1. AST-Parsed Languages (Tree-sitter 0.25)

| Language | Extensions | Symbols Extracted | Imports / Exports | Entry Points |
|---|---|---|---|---|
| **Rust** | `.rs` | `fn`, `struct`, `enum`, `trait`, `impl` methods, `mod` | `use`, `mod`, `pub use` | `fn main()`, `src/lib.rs`, `src/main.rs` |
| **JavaScript** | `.js`, `.mjs`, `.cjs`, `.jsx` | functions, classes, methods, arrow bindings | ES `import`/`export`, CommonJS `require` | `index.js`, `server.js`, `app.js` |
| **TypeScript / TSX** | `.ts`, `.tsx` | functions, classes, interfaces, methods, types | ES `import`/`export`, relative path resolution | `index.ts`, `server.ts`, `main.ts` |
| **Python** | `.py` | functions, classes, methods, decorated defs | `import`, `from ... import`, relative `.` / `..` | `if __name__ == "__main__":`, `app.py`, `wsgi.py`, `manage.py` |
| **Go** | `.go` | funcs, methods (with receivers), structs, interfaces | single & block `import (...)` | `func main()` in `package main` |
| **Java** | `.java` | classes, interfaces, enums, methods, constructors | package & `import` statements | `public static void main` |
| **C** | `.c`, `.h` | functions, structs, enums | `#include "..."` and `<...>` | `int main(...)` |
| **C++** | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh` | functions, classes, structs, enums, namespaces | `#include "..."` and `<...>` | `int main(...)` |

*(Plus 17 additional languages recognized at the repository inventory level in Stage 01: C#, Ruby, PHP, Swift, Kotlin, Dart, Scala, Shell, SQL, HTML, CSS, YAML, JSON, TOML, Markdown, XML, Dockerfile.)*

---

## 2. API Route & Endpoint Detectors (`src/structure/routes.rs`)

- **JavaScript / TypeScript:** Express, Fastify, Koa (`app.get`, `router.post`, `.use(...)`)
- **Next.js:** Pages API (`pages/api/**/*.ts`) and App Router (`app/api/**/route.ts` exported `GET`/`POST`/`PUT`/`DELETE`/`PATCH`)
- **Python:** FastAPI & APIRouter (`@app.get`, `@router.post`), Flask (`@app.route(..., methods=[...])`)
- **Go:** Gin & Echo (`r.GET("/path", handler)`)
- **Rust:** Axum & Actix-web (`.route("/path", ...)`, `#[get("/path")]`)
- **Java:** Spring Boot (`@GetMapping`, `@PostMapping`, `@PutMapping`, `@DeleteMapping`, `@PatchMapping`, `@RequestMapping`)

---

## 3. Persistence & Database Detectors (`src/structure/datamodels.rs`)

- **Python ORMs:** Django (`models.Model`), SQLAlchemy (`Base`)
- **TypeScript ORMs:** TypeORM (`@Entity()`), Prisma (`schema.prisma` `model X`)
- **Relational SQL:** Migration scripts (`migrations/`, `db/`, `sql/*.sql`) and standalone `CREATE TABLE` DDL definitions
