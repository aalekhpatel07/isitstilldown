from dataclasses import dataclass
from dataclasses_json import dataclass_json
from datetime import timedelta


@dataclass_json
@dataclass(frozen=True, order=True, eq=True)
class TimeSinceEpoch:
    """Representation of the SystemTime struct from Rust."""
    secs_since_epoch: int
    nanos_since_epoch: int

    def as_str(self) -> str:
        return str(self.secs_since_epoch * 1_000_000_000 + self.nanos_since_epoch)


@dataclass_json
@dataclass(frozen=True, order=True, eq=True)
class Duration:
    """Class that represents the Duration struct from Rust."""
    secs: int
    nanos: int

    def as_str(self) -> str:
        return str(self.secs * 1_000_000_000 + self.nanos)


@dataclass_json
@dataclass(frozen=True, order=True, eq=True)
class Resource:
    last_ping_at: TimeSinceEpoch
    url: str
    active: bool


@dataclass_json
@dataclass(frozen=True, order=True, eq=True)
class PingEvent:
    """Class that represents a PingEvent generated from domain-watcher."""
    resource: Resource
    request_init: TimeSinceEpoch
    response_time: Duration
    response_code: int