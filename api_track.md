# Routes
## /users
- [x] POST `/register`
- [x] POST `/refresh`
- [x] POST `/login`
- [x] POST `/update`(alias, email)
- [x] GET `/me`
- [x] POST `/change_password`
## /recipes
- [ ] `/` GET get (maybe paginate later)
- [ ] `/` POST (name, ingredients(with unit count))
- [ ] `/:id` GET detail (with ingredients) (maybe paginate later)
- [ ] `/:id` UPDATE (name, ingredients(with unit count))
- [ ] `/:id` DELETE
## /ingredients
- [ ] `/` GET (maybe pageinate later, but hard with sorting)
- [ ] `/` POST (name, unit)
- [ ] `/:id` UPDATE (name, unit)
- [ ] `/:id` DELETE
## /units
- [ ] `/` GET
