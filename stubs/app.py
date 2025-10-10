from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional, List

app = FastAPI()


class Event(BaseModel):
    id: int
    name: str
    description: str
    dateOfEvent: str


class Category(BaseModel):
    id: int
    name: str
    description: Optional[str]


class Track(BaseModel):
    id: int
    name: str
    description: str


class Racer(BaseModel):
    id: int
    firstName: str
    lastName: str
    startNumber: Optional[int]
    categories: List[Category]
    tagId: Optional[str]
    track: Track


class Result(BaseModel):
    registrationId: int
    startTime: str
    finishTime: str


class RaceResult(BaseModel):
    results: List[Result]


@app.get("/races")
def races() -> List[Event]:
    return [
        Event(
            id=1,
            name="Přespolňák 2019",
            description="Již čtvrtý ročník oblíbeného přespolního běhu",
            dateOfEvent="2019-10-05 10:00:00.000000",
        ),
        Event(
            id=2,
            name="Přespolňák 2020",
            description="Již paty ročník oblíbeného přespolního běhu",
            dateOfEvent="2020-10-05 10:00:00.000000",
        ),
        Event(
            id=3,
            name="Přespolňák 2021",
            description="Již sesty ročník oblíbeného přespolního běhu",
            dateOfEvent="2021-10-05 10:00:00.000000",
        ),
    ]


@app.get("/races/{race_id}/registrations")
def registrations(race_id: int) -> List[Racer]:
    return [
        Racer(
            id=1,
            firstName="John",
            lastName="Doe",
            startNumber=None,
            categories=[Category(id=92, name="A10", description=None)],
            tagId="1234",
            track=Track(id=25, name="10 Km", description=""),
        ),
        Racer(
            id=2,
            firstName="Carlos",
            lastName="Smith",
            startNumber=123,
            categories=[Category(id=92, name="A10", description=None)],
            tagId="1111",
            track=Track(id=25, name="10 Km", description=""),
        ),
        Racer(
            id=3,
            firstName="Oioioi",
            lastName="Boi",
            startNumber=124,
            categories=[Category(id=93, name="B4", description=None)],
            tagId=None,
            track=Track(id=25, name="4 Km", description=""),
        ),
    ]


@app.post("/races/{race_id}/results")
def results(race_id: int, results: RaceResult):
    print(results)
    return {}
