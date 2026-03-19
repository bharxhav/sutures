use sutures::Seam;
use sutures::seam::{SeamField, SeamFieldType};

// ============================================================================
// Basic struct tests
// ============================================================================

#[derive(Seam)]
struct BasicStruct {
    name: String,
    age: u32,
    active: bool,
}

#[test]
fn basic_struct_fields() {
    let fields = BasicStruct::fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "name");
    assert_eq!(fields[1].name, "age");
    assert_eq!(fields[2].name, "active");
}

#[test]
fn basic_struct_is_not_enum() {
    assert!(!BasicStruct::IS_ENUM);
    assert!(!BasicStruct::IS_ANON_STRUCT);
}

#[test]
fn basic_struct_all_terminal() {
    for field in BasicStruct::fields() {
        assert!(matches!(field.ty, SeamFieldType::Terminal));
    }
}

// ============================================================================
// Empty struct
// ============================================================================

#[derive(Seam)]
struct EmptyStruct {}

#[test]
fn empty_struct_fields() {
    assert_eq!(EmptyStruct::fields().len(), 0);
}

// ============================================================================
// Single field struct
// ============================================================================

#[derive(Seam)]
struct SingleField {
    value: i64,
}

#[test]
fn single_field_struct() {
    let fields = SingleField::fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "value");
}

// ============================================================================
// Skip attribute
// ============================================================================

#[derive(Seam)]
struct WithSkip {
    visible: String,
    #[seam(skip)]
    hidden: u64,
    also_visible: bool,
}

#[test]
fn skip_excludes_field() {
    let fields = WithSkip::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "visible");
    assert_eq!(fields[1].name, "also_visible");
}

// ============================================================================
// All fields skipped
// ============================================================================

#[derive(Seam)]
struct AllSkipped {
    #[seam(skip)]
    a: i32,
    #[seam(skip)]
    b: i32,
}

#[test]
fn all_fields_skipped() {
    assert_eq!(AllSkipped::fields().len(), 0);
}

// ============================================================================
// Rename attribute
// ============================================================================

#[derive(Seam)]
struct WithRename {
    #[seam(rename = "type")]
    ty: String,
    #[seam(rename = "model_id")]
    id: u64,
}

#[test]
fn rename_changes_field_name() {
    let fields = WithRename::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "type");
    assert_eq!(fields[1].name, "model_id");
}

// ============================================================================
// Raw identifier
// ============================================================================

#[derive(Seam)]
struct WithRawIdent {
    r#type: String,
}

#[test]
fn raw_ident_strips_prefix() {
    let fields = WithRawIdent::fields();
    assert_eq!(fields.len(), 1);
    // syn::Ident::to_string() strips the r# prefix
    assert_eq!(fields[0].name, "type");
}

// ============================================================================
// Nested struct (to_struct)
// ============================================================================

#[derive(Seam)]
struct Inner {
    x: f64,
    y: f64,
}

#[derive(Seam)]
struct Outer {
    name: String,
    #[seam(to_struct)]
    position: Inner,
}

#[test]
fn to_struct_produces_struct_variant() {
    let fields = Outer::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "name");
    assert!(matches!(fields[0].ty, SeamFieldType::Terminal));

    assert_eq!(fields[1].name, "position");
    match fields[1].ty {
        SeamFieldType::Struct(f) => {
            let inner_fields = f();
            assert_eq!(inner_fields.len(), 2);
            assert_eq!(inner_fields[0].name, "x");
            assert_eq!(inner_fields[1].name, "y");
        }
        _ => panic!("expected SeamFieldType::Struct"),
    }
}

#[test]
fn to_struct_const_checks() {
    // Inner is not an enum and not an anon struct — const asserts pass.
    assert!(!Inner::IS_ENUM);
    assert!(!Inner::IS_ANON_STRUCT);
}

// ============================================================================
// Nested enum (to_enum)
// ============================================================================

#[derive(Seam)]
enum Status {
    Active,
    Inactive,
    Custom { code: u32, label: String },
}

#[derive(Seam)]
struct WithEnum {
    name: String,
    #[seam(to_enum)]
    status: Status,
}

#[test]
fn to_enum_produces_enum_variant() {
    let fields = WithEnum::fields();
    assert_eq!(fields.len(), 2);

    assert_eq!(fields[1].name, "status");
    match fields[1].ty {
        SeamFieldType::Enum(f) => {
            let variants = f();
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Active");
            assert_eq!(variants[1].name, "Inactive");
            assert_eq!(variants[2].name, "Custom");
        }
        _ => panic!("expected SeamFieldType::Enum"),
    }
}

#[test]
fn enum_is_enum() {
    assert!(Status::IS_ENUM);
    assert!(!Status::IS_ANON_STRUCT);
}

// ============================================================================
// Enum tests
// ============================================================================

#[derive(Seam)]
enum BasicEnum {
    A,
    B,
    C,
}

#[test]
fn basic_enum_variants() {
    let fields = BasicEnum::fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "A");
    assert_eq!(fields[1].name, "B");
    assert_eq!(fields[2].name, "C");
    for field in fields {
        assert!(matches!(field.ty, SeamFieldType::Terminal));
    }
}

#[derive(Seam)]
enum EmptyEnum {}

#[test]
fn empty_enum_fields() {
    assert_eq!(EmptyEnum::fields().len(), 0);
}

// ============================================================================
// Enum with named variant (AnonymousStruct)
// ============================================================================

#[derive(Seam)]
enum MixedEnum {
    Unit,
    Tuple(i32, String),
    Named { x: f64, y: f64 },
}

#[test]
fn enum_variant_types() {
    let fields = MixedEnum::fields();
    assert_eq!(fields.len(), 3);

    // Unit variant
    assert_eq!(fields[0].name, "Unit");
    assert!(matches!(fields[0].ty, SeamFieldType::Terminal));

    // Tuple variant — Terminal (serde handles)
    assert_eq!(fields[1].name, "Tuple");
    assert!(matches!(fields[1].ty, SeamFieldType::Terminal));

    // Named variant — AnonymousStruct
    assert_eq!(fields[2].name, "Named");
    match fields[2].ty {
        SeamFieldType::AnonymousStruct(f) => {
            let children = f();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].name, "x");
            assert_eq!(children[1].name, "y");
        }
        _ => panic!("expected SeamFieldType::AnonymousStruct"),
    }
}

// ============================================================================
// Enum with skip and rename on variants
// ============================================================================

#[derive(Seam)]
enum EnumWithAttrs {
    #[seam(rename = "active")]
    Active,
    #[seam(skip)]
    Internal,
    Inactive,
}

#[test]
fn enum_skip_and_rename() {
    let fields = EnumWithAttrs::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "active");
    assert_eq!(fields[1].name, "Inactive");
}

// ============================================================================
// Deep nesting — struct -> struct -> struct
// ============================================================================

#[derive(Seam)]
struct Coord {
    lat: f64,
    lon: f64,
}

#[derive(Seam)]
struct Address {
    street: String,
    #[seam(to_struct)]
    coord: Coord,
}

#[derive(Seam)]
struct User {
    name: String,
    #[seam(to_struct)]
    address: Address,
}

#[test]
fn deep_nesting() {
    let fields = User::fields();
    assert_eq!(fields.len(), 2);

    // Drill into address
    let addr_fields = match fields[1].ty {
        SeamFieldType::Struct(f) => f(),
        _ => panic!("expected Struct"),
    };
    assert_eq!(addr_fields.len(), 2);
    assert_eq!(addr_fields[0].name, "street");

    // Drill into coord
    let coord_fields = match addr_fields[1].ty {
        SeamFieldType::Struct(f) => f(),
        _ => panic!("expected Struct"),
    };
    assert_eq!(coord_fields.len(), 2);
    assert_eq!(coord_fields[0].name, "lat");
    assert_eq!(coord_fields[1].name, "lon");
}

// ============================================================================
// Enum with to_struct inside named variant
// ============================================================================

#[derive(Seam)]
enum EnumWithNestedStruct {
    Simple,
    Complex {
        label: String,
        #[seam(to_struct)]
        position: Coord,
    },
}

#[test]
fn enum_variant_with_to_struct() {
    let fields = EnumWithNestedStruct::fields();
    assert_eq!(fields.len(), 2);

    match fields[1].ty {
        SeamFieldType::AnonymousStruct(f) => {
            let children = f();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].name, "label");
            assert!(matches!(children[0].ty, SeamFieldType::Terminal));

            assert_eq!(children[1].name, "position");
            match children[1].ty {
                SeamFieldType::Struct(g) => {
                    let coord = g();
                    assert_eq!(coord.len(), 2);
                    assert_eq!(coord[0].name, "lat");
                }
                _ => panic!("expected Struct inside AnonymousStruct"),
            }
        }
        _ => panic!("expected AnonymousStruct"),
    }
}

// ============================================================================
// Generics — struct with lifetime
// ============================================================================

#[derive(Seam)]
struct WithLifetime<'a> {
    name: &'a str,
    value: i32,
}

#[test]
fn lifetime_struct() {
    let fields = WithLifetime::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "name");
    assert_eq!(fields[1].name, "value");
}

// ============================================================================
// Generics — struct with type param
// ============================================================================

#[derive(Seam)]
struct Generic<T> {
    value: T,
    label: String,
}

#[test]
fn generic_struct() {
    let fields = Generic::<i32>::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "value");
    assert_eq!(fields[1].name, "label");
}

// ============================================================================
// Generics — struct with where clause
// ============================================================================

#[derive(Seam)]
struct WithWhere<T>
where
    T: Send,
{
    data: T,
}

#[test]
fn generic_with_where_clause() {
    let fields = WithWhere::<String>::fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "data");
}

// ============================================================================
// Generics — enum with type param
// ============================================================================

#[derive(Seam)]
enum GenericEnum<T> {
    Some(T),
    None,
}

#[test]
fn generic_enum() {
    assert!(GenericEnum::<i32>::IS_ENUM);
    let fields = GenericEnum::<i32>::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "Some");
    assert_eq!(fields[1].name, "None");
}

// ============================================================================
// Generics — multiple type params and lifetime
// ============================================================================

#[derive(Seam)]
struct MultiGeneric<'a, T, U> {
    reference: &'a str,
    first: T,
    second: U,
}

#[test]
fn multi_generic() {
    let fields = MultiGeneric::<i32, String>::fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "reference");
    assert_eq!(fields[1].name, "first");
    assert_eq!(fields[2].name, "second");
}

// ============================================================================
// Std lib type coverage — all common types as Terminal fields
// ============================================================================

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Seam)]
struct StdLibTypes {
    // Primitives
    a_bool: bool,
    a_char: char,
    a_i8: i8,
    a_i16: i16,
    a_i32: i32,
    a_i64: i64,
    a_u8: u8,
    a_u16: u16,
    a_u32: u32,
    a_u64: u64,
    a_f32: f32,
    a_f64: f64,
    a_usize: usize,
    a_isize: isize,

    // String types
    a_string: String,

    // Smart pointers
    a_box: Box<i32>,
    a_arc: Arc<String>,

    // Option and Result
    a_option: Option<i32>,
    a_result: Result<String, i32>,

    // Collections
    a_vec: Vec<i32>,
    a_vecdeque: VecDeque<String>,
    a_hashmap: HashMap<String, i32>,
    a_btreemap: BTreeMap<String, i32>,
    a_hashset: HashSet<String>,
    a_btreeset: BTreeSet<i32>,

    // Tuples
    a_tuple2: (i32, String),
    a_tuple3: (bool, f64, String),

    // Arrays
    a_array: [u8; 32],

    // Nested generics
    a_nested: Vec<Option<Box<HashMap<String, Vec<i32>>>>>,
}

#[test]
fn std_lib_types_all_terminal() {
    let fields = StdLibTypes::fields();
    assert_eq!(fields.len(), 29);
    for field in fields {
        assert!(
            matches!(field.ty, SeamFieldType::Terminal),
            "field {} should be Terminal",
            field.name
        );
    }
}

// ============================================================================
// Field ordering — fields maintain declaration order
// ============================================================================

#[derive(Seam)]
struct Ordered {
    zebra: String,
    alpha: String,
    middle: String,
}

#[test]
fn field_order_matches_declaration() {
    let fields = Ordered::fields();
    assert_eq!(fields[0].name, "zebra");
    assert_eq!(fields[1].name, "alpha");
    assert_eq!(fields[2].name, "middle");
}

// ============================================================================
// Visibility — pub/private fields both work
// ============================================================================

#[derive(Seam)]
pub struct WithVisibility {
    pub public_field: String,
    private_field: i32,
    pub(crate) crate_field: bool,
}

#[test]
fn visibility_does_not_affect_fields() {
    let fields = WithVisibility::fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "public_field");
    assert_eq!(fields[1].name, "private_field");
    assert_eq!(fields[2].name, "crate_field");
}
