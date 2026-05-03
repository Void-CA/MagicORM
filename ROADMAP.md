# MagicORM — Roadmap y Plan de Desarrollo

## Estado Actual (resumen)
- Rama principal: `main`
- Rama de desarrollo activa: `eager_loading` (funcional pero incompleta)
- Soporte actual: Solo SQLite (el código tiene hardcoding, Postgres no compila)
- Eager loading: Implementado para `has_many`, falta `belongs_to`
- Transacciones: Rotas (CRUD usa `&Pool<Sqlite>`, no genérico)
- Documentación: README desactualizado, TODO con items de producción (Debita)

---

## Tareas Críticas a Corto Plazo (priorizadas)

1. **Refactor estructural (basado en `tasks/refactor.md`)**
   - Unificar el concepto `Model` fragmentado en `meta/`, `query/`, `schema/`, `traits/` en un solo módulo `model/` claro.
   - Separar código de compile-time (derive macros) de runtime.
   - Limpiar `relations/` para no mezclar macros, traits y lógica de runtime.

2. **Corregir fallas de diseño en Eager Loading**
   - Hacer que `EagerQueryBuilder` delegue todos los métodos de `QueryBuilder` (filtros, orden, límites) para que el usuario pueda encadenar operaciones antes de cargar relaciones.
   - Eliminar hardcoding a `i64` para IDs: hacer el tipo de ID genérico en `Model`, `HasFK` y todos los traits relacionados.
   - Eliminar hardcoding a SQLite: reemplazar usos de `sqlx::sqlite::SqliteRow` por traits genéricos de sqlx (`sqlx::FromRow` genérico, `sqlx::Executor` genérico).
   - Implementar eager loading para `belongs_to` para cubrir el mismo alcance que `has_many`.

3. **Arreglar soporte a transacciones (TODO Debita)**
   - Reemplazar `&Pool<Sqlite>` por `sqlx::Executor<'_, Database = DB>` genérico en todas las funciones CRUD y de query. Esto permite usar el ORM dentro de transacciones de sqlx.

4. **Documentación honesta**
   - Actualizar `README.md`: eliminar la mención a "fase de diseño", documentar estado real, features soportados (solo SQLite por ahora), limitaciones y ejemplos de uso.
   - Actualizar `TODO.md`: marcar items completados, agregar nuevos puntos de refactor y correcciones.

5. **Higiene de código**
   - Agregar `test.db` y `*.db` a `.gitignore` (no commitear binarios de base de datos).
   - Limpiar warnings en el core (ignorar `magic_cli` según decisión del equipo).

---

## Plan de Desarrollo (Fases)

### Fase 1: Cimientos Sólidos (OBLIGATORIA antes de nuevas features)
> **No agregar ninguna feature nueva (Postgres, views, etc.) hasta terminar esta fase**
- Ejecutar el refactor de `tasks/refactor.md` al pie de la letra.
- Resolver todos los puntos de la sección "Tareas Críticas" arriba.
- Actualizar documentación (`README.md`, `TODO.md`).
- Limpiar código y commitear cambios en `eager_loading`.
- **NO MERGEAR `eager_loading` a `main` hasta que esta fase esté 100% completa**.

### Fase 2: Estabilización
- Implementar partial updates (punto del TODO de Debita).
- Evaluar separación de domain/persistence model (opcional, solo si aporta valor real).
- Agregar tests unitarios e integración para:
  - CRUD básico
  - Lazy y eager loading (`has_many` y `belongs_to`)
  - Transacciones
- Cubrir casos borde (IDs vacíos en eager loading, transacciones fallidas).

### Fase 3: Nuevas Features (solo después de Fase 1 y 2)
- **Soporte real a Postgres**:
  - Usar feature flags de sqlx correctamente (`sqlx/postgres` en `Cargo.toml`).
  - Probar ambas DBs en CI, no asumir sintaxis de SQLite (ej. placeholders `?` vs `$1` en Postgres).
  - Verificar compatibilidad de tipos de datos.
- **Soporte a vistas**:
  - Definir si las vistas son read-only.
  - Mecanismo para mapear vistas a structs del ORM sin romper la abstracción de `Model`.
- Merge de `eager_loading` a `main` cuando todo esté estable.

---

## Consideraciones para Futuros Features

### Antes de Postgres:
- Todos los puntos de la Fase 1 son obligatorios. Si intentás agregar Postgres ahora mismo:
  - El hardcoding a SQLite hará que el código no compile.
  - El refactor de `Model` será 10x más difícil con eager loading de por medio.
  - Las transacciones siguen rotas.

### Antes de Views:
- Definir claramente el alcance: ¿son vistas de solo lectura? ¿se pueden escribir?
- No romper la abstracción actual de `Model`: las vistas deberían ser un caso especial de lectura, no forzarlas a implementar toda la interfaz de escritura.

### General:
- Siempre preferir abstracciones genéricas sobre hardcodings a una base de datos específica.
- Mantener la filosofía de "conventions over configuration" que define al ORM.
- No agregar features hasta que las existentes estén estables.

---

## Correcciones Pendientes (mencionadas en revisión técnica)

1. `EagerQueryBuilder` no delega métodos de `QueryBuilder` → el usuario no puede filtrar padres antes de cargar hijos.
2. Acoplamiento a `i64` para IDs → no soporta UUIDs u otros tipos de claves primarias.
3. Acoplamiento a `sqlx::sqlite::SqliteRow` → el código no compila con Postgres.
4. Construcción manual de `WHERE IN` en eager loading → inconsistente con el `QueryBuilder` ya existente, usar el builder propio.
5. Falta eager loading para `belongs_to` (solo tiene implementación lazy).
6. `test.db` commiteado → debe estar en `.gitignore`.
7. README desactualizado → dice "fase de diseño" incorrectamente.
