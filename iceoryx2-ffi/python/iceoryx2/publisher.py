from typing import Generic, TypeVar
from . import _iceoryx2

T = TypeVar("T")

class Publisher(Generic[T]):
    def __init__(self):
        print("init")
