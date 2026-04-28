#[cynic::schema("linear")]
pub mod schema {}

impl cynic::schema::IsScalar<schema::DateTime> for String {
    type SchemaType = schema::DateTime;
}
impl cynic::coercions::CoercesTo<schema::DateTime> for String {}

impl cynic::schema::IsScalar<schema::TimelessDate> for String {
    type SchemaType = schema::TimelessDate;
}
impl cynic::coercions::CoercesTo<schema::TimelessDate> for String {}
