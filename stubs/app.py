from fastapi import FastAPI, Request

app = FastAPI()


@app.get("/races")
def races():
    return [
        {
            "id": 1,
            "name": "Přespolňák 2019",
            "description": "Již čtvrtý ročník oblíbeného přespolního běhu",
            "dateOfEvent": "2019-10-05 10:00:00.000000",
        },
        {
            "id": 2,
            "name": "Přespolňák 2020",
            "description": "Již paty ročník oblíbeného přespolního běhu",
            "dateOfEvent": "2020-10-05 10:00:00.000000",
        },
        {
            "id": 3,
            "name": "Přespolňák 2021",
            "description": "Již sesty ročník oblíbeného přespolního běhu",
            "dateOfEvent": "2021-10-05 10:00:00.000000",
        },
    ]


@app.get("/races/{race_id}/registrations")
def registrations(race_id: int):
    return [
        {
            "id": 1,
            "firstName": "John",
            "lastName": "Doe",
            "startNumber": None,
            "categories": [
                {
                    "id": 92,
                    "name": "A10",
                    "description": None,
                }
            ],
            "tagId": None,
            "track": {"id": 25, "name": "10 Km", "description": ""},
        },
        {
            "id": 2,
            "firstName": "Carlos",
            "lastName": "Smith",
            "startNumber": 123,
            "categories": [
                {
                    "id": 92,
                    "name": "A10",
                    "description": None,
                }
            ],
            "tagId": None,
            "track": {"id": 25, "name": "10 Km", "description": ""},
        },
        {
            "id": 3,
            "firstName": "Oioioi",
            "lastName": "Boi",
            "startNumber": 124,
            "categories": [
                {
                    "id": 93,
                    "name": "B4",
                    "description": None,
                }
            ],
            "tagId": None,
            "track": {"id": 25, "name": "4 Km", "description": ""},
        },
    ]


@app.post("/races/{race_id}/results")
async def results(race_id: int, request: Request):
    data = await request.json()
    print(data)
    return {}
