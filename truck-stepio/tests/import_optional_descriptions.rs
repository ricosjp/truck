use std::str::FromStr;
use truck_stepio::r#in::{
    ruststep::{ast::DataSection, tables::EntityTable},
    step_geometry::*,
    *,
};

const ASSEMBLY: &str = "DATA;
#1=PRODUCT('child','child',$,(#91));
#2=PRODUCT_DEFINITION_FORMATION('v1',$,#1);
#4=PRODUCT_DEFINITION('design',$,#2,#92);
#5=PRODUCT_DEFINITION_SHAPE('',$,#4);
#6=SHAPE_DEFINITION_REPRESENTATION(#5,#7);
#7=SHAPE_REPRESENTATION('',(#8),#93);
#8=AXIS2_PLACEMENT_3D('',#9,$,$);
#9=CARTESIAN_POINT('',(0.,0.,0.));
#11=PRODUCT('parent','parent',$,(#91));
#12=PRODUCT_DEFINITION_FORMATION('v1',$,#11);
#14=PRODUCT_DEFINITION('design',$,#12,#92);
#15=PRODUCT_DEFINITION_SHAPE('',$,#14);
#16=SHAPE_DEFINITION_REPRESENTATION(#15,#17);
#17=SHAPE_REPRESENTATION('',(#18),#93);
#18=AXIS2_PLACEMENT_3D('',#19,$,$);
#19=CARTESIAN_POINT('',(10.,0.,0.));
#20=NEXT_ASSEMBLY_USAGE_OCCURRENCE('instance','',$,#14,#4,$);
#21=PRODUCT_DEFINITION_SHAPE('',$,#20);
#22=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#23,#21);
#23=(REPRESENTATION_RELATIONSHIP('',$,#7,#17)
REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#24)
SHAPE_REPRESENTATION_RELATIONSHIP());
#24=ITEM_DEFINED_TRANSFORMATION('',$,#8,#18);
#90=APPLICATION_CONTEXT('test');
#91=PRODUCT_CONTEXT('',#90,'mechanical');
#92=PRODUCT_DEFINITION_CONTEXT('',#90,'design');
#93=REPRESENTATION_CONTEXT('','3D');
ENDSEC;";

fn strict_table(source: &str) -> std::result::Result<Table, String> {
    let data = DataSection::from_str(source).unwrap();
    let mut table = Table::default();
    for entity in data.entities {
        table.push_instance(&entity).map_err(|e| e.to_string())?;
    }
    Ok(table)
}

#[test]
fn omitted_descriptions_do_not_drop_assembly_nodes_or_edges() {
    let table = strict_table(ASSEMBLY).unwrap();
    assert_eq!(table.product.len(), 2);
    assert_eq!(table.product_definition.len(), 2);
    assert_eq!(table.product_definition_shape.len(), 3);
    assert_eq!(
        table
            .shape_representation_relationship_with_transformation
            .len(),
        1
    );
    let assembly = table.step_assy().unwrap();
    assert_eq!(assembly.len(), 2);
    assert_eq!(assembly.top_nodes().count(), 1);
    let top = assembly.top_nodes().next().unwrap();
    assert_eq!(assembly.maximal_paths_iter(top.index()).count(), 1);
    assert_eq!(top.attrs().description, "");
    let transform = EntityTable::<ItemDefinedTransformationHolder>::get_owned(&table, 24).unwrap();
    let matrix = Matrix4::try_from(&transform).unwrap();
    assert_eq!(
        matrix.transform_point(Point3::origin()),
        Point3::new(10., 0., 0.)
    );
}

#[test]
fn present_descriptions_are_preserved() {
    let source = ASSEMBLY
        .replace("$", "'description'")
        .replace("#9,'description','description'", "#9,$,$")
        .replace("#19,'description','description'", "#19,$,$");
    let table = strict_table(&source).unwrap();
    let assembly = table.step_assy().unwrap();
    assert_eq!(
        assembly.top_nodes().next().unwrap().attrs().description,
        "description"
    );
}

#[test]
fn specified_source_formation_accepts_omitted_description() {
    let table = strict_table("DATA; #1=PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE('v1',$,#2,.NOT_KNOWN.); ENDSEC;").unwrap();
    assert_eq!(table.product_definition_formation.len(), 1);
}

#[test]
fn mandatory_product_name_remains_required() {
    assert!(strict_table("DATA; #1=PRODUCT('id',$,'',(#2)); ENDSEC;").is_err());
}

#[test]
fn non_string_description_remains_an_error() {
    assert!(strict_table("DATA; #1=PRODUCT('id','name',42,(#2)); ENDSEC;").is_err());
}
