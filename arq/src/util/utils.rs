use uuid::Uuid;

pub trait HasUuid {
    fn get_id(&self) -> Uuid;
}

pub trait UuidEquals<B : HasUuid>:HasUuid {
    fn uuid_equals(&self, other: B) -> bool {
        return self.get_id() == other.get_id();
    }
}
