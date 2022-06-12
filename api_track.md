# Routes
## /users
- [x] POST `/register`
- [x] PATCH `/refresh`
- [x] POST `/login`
- [x] PUT `/update`(alias, email)
- [x] GET `/me`
- [x] PUT `/change_password`
## /recipes
- [x] GET `/` (maybe paginate later)
- [x] POST `/` (name, ingredients(with unit quantity))
- [x] GET `/:id` detail (with ingredients)
- [x] PUT `/:id` UPDATE (name, ingredients(with unit count))
- [x] DELETE `/:id`
## /ingredients
- [x] `/` GET (maybe pageinate later, but hard with sorting)
- [x] `/` POST (name, unit)
- [x] `/` PATCH (sort) (id: new position)
- [x] `/:id` UPDATE (name, unit)
- [x] `/:id` DELETE
## /units
- [x] `/` GET
