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
- [ ] PUT `/:id` UPDATE (name, ingredients(with unit count))
- [ ] DELETE `/:id`
## /ingredients
- [ ] `/` GET (maybe pageinate later, but hard with sorting)
- [ ] `/` POST (name, unit)
- [ ] `/:id` UPDATE (name, unit)
- [ ] `/:id` DELETE
## /units
- [ ] `/` GET
