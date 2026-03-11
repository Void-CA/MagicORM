# Observaciones desde Debita

## Transacciones
Magic CRUD functions require &Pool<Sqlite>.
This prevents usage inside sqlx transactions.

Proposed improvement:
Use generic Executor trait.

## Update API
Currently updates require rebuilding the model.
Consider partial updates. (patch)

## Domain separation
Investigate possibility of separating domain and persistence models. (or not)