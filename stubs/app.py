from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional, List
import random

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
    racers_count = 100
    firstnames = [
        "John",
        "Jane",
        "Alice",
        "Bob",
        "Tom",
        "Sara",
        "Mike",
        "Emma",
        "Liam",
        "Olivia",
    ]
    lastnames = [
        "Doe",
        "Smith",
        "Brown",
        "Johnson",
        "Williams",
        "Jones",
        "Davis",
        "Miller",
        "Wilson",
        "Taylor",
    ]
    tracks = [
        Track(id=1, name="10 Km", description=""),
        Track(id=2, name="4 Km", description=""),
        Track(id=3, name="Dedska trat", description=""),
    ]
    categories = [
        Category(id=1, name="A10", description=None),
        Category(id=2, name="B4", description=None),
        Category(id=3, name="C4", description=None),
        Category(id=4, name="D10", description=None),
        Category(id=5, name="A10", description=None),
        Category(id=6, name="J4", description=None),
        Category(id=7, name="Deti", description=None),
    ]

    random.seed(race_id)
    racers = []
    for i in range(racers_count):
        num_categories = random.choices([1, 2, 3], weights=[70, 20, 10], k=1)[0]
        racers.append(
            Racer(
                id=random.randint(0, 10000),
                firstName=random.choice(firstnames),
                lastName=random.choice(lastnames),
                startNumber=i + 1,
                categories=random.choices(categories, k=num_categories),
                tagId="".join(random.choices("0123456789ABCDEF", k=8)),
                track=random.choice(tracks),
            )
        )
    return racers


@app.post("/races/{race_id}/results")
def results(race_id: int, results: RaceResult):
    print(results)
    return {}
