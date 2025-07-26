#![doc = include_str!("../README.md")]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::*;
use syn::*;

fn enumerate_impl_return_something<'a>(
    variants: impl IntoIterator<Item = &'a Variant> + 'a,
    method: TokenStream2,
    method_variants: TokenStream2,
) -> TokenStream2 {
    let impls: Vec<_> = variants
        .into_iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let vec: Vec<_> = variant.fields.iter().collect();
            match vec.len() {
                0 => panic!("empty field!"),
                1 => match &vec[0].ident {
                    Some(ident) => quote! {
                        Self::#variant_name { #ident } => #method(#ident, #method_variants)
                    },
                    None => quote! {
                        Self::#variant_name(got) => #method(got, #method_variants)
                    },
                },
                _ => unimplemented!(),
            }
        })
        .collect();
    quote! { match self { #(#impls),* } }
}

fn enumerate_impl_return_self<'a>(
    variants: impl IntoIterator<Item = &'a Variant> + 'a,
    method: TokenStream2,
    method_variants: TokenStream2,
) -> TokenStream2 {
    let impls: Vec<_> = variants
        .into_iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let vec: Vec<_> = variant.fields.iter().collect();
            match vec.len() {
                0 => panic!("empty field!"),
                1 => match &vec[0].ident {
                    Some(ident) => {
                        quote! {
                            Self::#variant_name { #ident } => Self::#variant_name { #ident: #method(#ident, #method_variants) }
                        }
                    }
                    None => {
                        quote! {
                            Self::#variant_name(got) => Self::#variant_name(#method(got, #method_variants))
                        }
                    }
                },
                _ => unimplemented!(),
            }
        })
        .collect();
    quote! {
        match self {
            #(#impls),*
        }
    }
}

#[derive(Clone, Debug)]
struct Field {
    var: TokenStream2,
    ty: TokenStream2,
}

macro_rules! fields {
    ($($var: tt : $ty: tt),*) => {
        vec![$(Field {
            var: quote! { $var },
            ty: quote! { $ty },
        }),*]
    };
}

#[derive(Clone, Debug)]
struct Method<I> {
    name: TokenStream2,
    generics: Option<TokenStream2>,
    self_field: TokenStream2,
    fields: Vec<Field>,
    return_type: TokenStream2,
    variants: I,
    trait_name: TokenStream2,
}

macro_rules! methods {
    (
        $variants: ident, $trait_name: ident,
        $(fn $name: ident (
            $self_field: expr,
            $($var: ident: $ty: ty),*$(,)?
        ) -> $return_type: ty),*$(,)?
    ) => {
        vec![$(Method {
            name: quote! { $name },
            generics: None,
            self_field: quote! { $self_field, },
            fields: fields!($($var: $ty),*),
            return_type: quote! { $return_type },
            variants: $variants,
            trait_name: $trait_name.clone(),
        }
        .to_token_stream()),*]
    };
    (
        $variants: ident, $trait_name: ident,
        $(fn $name: ident <$($(($const: tt))? $gen: ident: $path: path),*> (
            $self_field: expr,
            $($var: ident: $ty: ty),*$(,)?
        ) -> $return_type: ty),*$(,)?
    ) => {
        vec![$(Method {
            name: quote! { $name },
            generics: Some(quote! { <$($($const)? $gen: $path),*> }),
            self_field: quote! { $self_field, },
            fields: fields!($($var: $ty),*),
            return_type: quote! { $return_type },
            variants: $variants,
            trait_name: $trait_name.clone(),
        }
        .to_token_stream()),*]
    };
}

impl<'a, I> Method<I>
where I: IntoIterator<Item = &'a Variant> + 'a + Copy
{
    fn to_token_stream(&'a self) -> TokenStream2 {
        let method_name = &self.name;
        let generics = &self.generics;
        let trait_name = &self.trait_name;
        let self_field = &self.self_field;
        let fields = self
            .fields
            .iter()
            .map(|f| {
                let var = &f.var;
                let ty = &f.ty;
                quote! { #var: #ty }
            })
            .collect::<Vec<_>>();
        let vals = self.fields.iter().map(|f| &f.var).collect::<Vec<_>>();
        let return_type = &self.return_type;
        let implement = if return_type.to_string() == "Self" {
            enumerate_impl_return_self::<'_>(
                self.variants,
                quote! { #trait_name::#method_name },
                quote! { #(#vals),* },
            )
        } else {
            enumerate_impl_return_something::<'a>(
                self.variants,
                quote! { #trait_name::#method_name },
                quote! { #(#vals),* },
            )
        };
        quote! {
            fn #method_name #generics (#self_field #(#fields),*) -> #return_type { #implement }
        }
    }
}

/// Derive macro generating an impl of the trait `BoundedCurve` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(BoundedCurve)]
pub fn derive_bounded_curve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::BoundedCurve };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant
                .fields
                .iter()
                .map(|field| &field.ty)
                .collect();
            let top_ty = &tys[0];
            let tys = &tys[1..];
            let methods = methods! {
                variants, trait_name,
                fn range_tuple(&self,) -> (f64, f64),
                fn front(&self,) -> Self::Point,
                fn back(&self,) -> Self::Point,
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #top_ty: #trait_name,
                    #(#tys: #trait_name<Point = <#top_ty as ParametricCurve>::Point>,)*
                    Self: truck_geotrait::ParametricCurve<Point = <#top_ty as ParametricCurve>::Point>, {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name,
                    Self: truck_geotrait::ParametricCurve<Point
                        = <#field_type as truck_geotrait::ParametricCurve>::Point>, {
                    fn range_tuple(&self) -> (f64, f64) { self.0.range_tuple() }
                    fn front(&self) -> Self::Point { self.0.front() }
                    fn back(&self) -> Self::Point { self.0.back() }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `BoundedSurface` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(BoundedSurface)]
pub fn derive_bounded_surface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::BoundedSurface };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods! {
                variants, trait_name,
                fn range_tuple(&self,) -> ((f64, f64), (f64, f64)),
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name {
                    fn range_tuple(&self) -> ((f64, f64), (f64, f64)) {
                        self.0.range_tuple()
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `Cut` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(Cut)]
pub fn derive_cut(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::Cut };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods! {
                variants, trait_name,
                fn cut(&mut self, t: f64) -> Self,
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name {
                    fn cut(&mut self, t: f64) -> Self { Self(self.0.cut(t)) }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `Invertible` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(Invertible)]
pub fn derive_invertible(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::Invertible };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods! {
                variants, trait_name,
                fn invert(&mut self,) -> (),
                fn inverse(&self,) -> Self,
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)*
                    Self: Clone {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name,
                    Self: Clone {
                    fn invert(&mut self) { self.0.invert() }
                    fn inverse(&self) -> Self { Self(self.0.inverse()) }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ParameterDivision1D` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(ParameterDivision1D)]
pub fn derive_parameter_division_1d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::ParameterDivision1D };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = match tys.len() {
                0 => panic!("empty field!"),
                1 => tys[0].clone(),
                _ => unimplemented!(),
            };
            let tys = &tys[1..];
            let methods = methods! {
                variants, trait_name,
                fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>),
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #top_ty: #trait_name,
                    #(#tys: #trait_name<Point = <#top_ty as #trait_name>::Point>,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name {
                    type Point = <#field_type as #trait_name>::Point;
                    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
                        self.0.parameter_division(range, tol)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ParameterDivision2D` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(ParameterDivision2D)]
pub fn derive_parameter_division_2d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::ParameterDivision2D };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods! {
                variants, trait_name,
                fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>),
            };
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name {
                    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>) {
                        self.0.parameter_division(range, tol)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ParametricCurve` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(ParametricCurve)]
pub fn derive_parametric_curve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::ParametricCurve };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let methods = methods!(
                variants,
                trait_name,
                fn subs(&self, t: f64) -> Self::Point,
                fn der(&self, t: f64) -> Self::Vector,
                fn der2(&self, t: f64) -> Self::Vector,
                fn der_n(&self, n: usize, t: f64) -> Self::Vector,
                fn ders(&self, n: usize, t: f64) -> CurveDers<Self::Vector>,
                fn parameter_range(&self,) -> ParameterRange,
                fn period(&self,) -> Option<f64>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where #(#where_predicates,)*
                      #(#tys: #trait_name,)*
                      Self: Clone {
                    type Point = <#top_ty as #trait_name>::Point;
                    type Vector = <#top_ty as #trait_name>::Vector;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where #(#where_predicates,)*
                      #field_type: #trait_name,
                      Self: Clone, {
                    type Point = <#field_type as #trait_name>::Point;
                    type Vector = <#field_type as #trait_name>::Vector;
                    fn subs(&self, t: f64) -> Self::Point { self.0.subs(t) }
                    fn der(&self, t: f64) -> Self::Vector { self.0.der(t) }
                    fn der2(&self, t: f64) -> Self::Vector { self.0.der2(t) }
                    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.0.der_n(n, t) }
                    fn ders(&self, n: usize, t: f64) -> CurveDers<Self::Vector> { self.0.ders(n, t) }
                    fn parameter_range(&self) -> ParameterRange { self.0.parameter_range() }
                    fn period(&self) -> Option<f64> { self.0.period() }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ParametricSurface` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(ParametricSurface)]
pub fn derive_parametric_surface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::ParametricSurface };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let methods = methods!(
                variants,
                trait_name,
                fn subs(&self, s: f64, t: f64) -> Self::Point,
                fn uder(&self, s: f64, t: f64) -> Self::Vector,
                fn vder(&self, s: f64, t: f64) -> Self::Vector,
                fn uuder(&self, s: f64, t: f64) -> Self::Vector,
                fn uvder(&self, s: f64, t: f64) -> Self::Vector,
                fn vvder(&self, s: f64, t: f64) -> Self::Vector,
                fn der_mn(&self, m: usize, n: usize, s: f64, t: f64) -> Self::Vector,
                fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector>,
                fn parameter_range(&self,) -> (truck_geotrait::ParameterRange, truck_geotrait::ParameterRange),
                fn u_period(&self,) -> Option<f64>,
                fn v_period(&self,) -> Option<f64>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    type Vector = <#top_ty as #trait_name>::Vector;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    type Point = <#field_type as #trait_name>::Point;
                    type Vector = <#field_type as #trait_name>::Vector;
                    #[inline(always)]
                    fn subs(&self, s: f64, t: f64) -> Self::Point { self.0.subs(s, t) }
                    #[inline(always)]
                    fn uder(&self, s: f64, t: f64) -> Self::Vector { self.0.uder(s, t) }
                    #[inline(always)]
                    fn vder(&self, s: f64, t: f64) -> Self::Vector { self.0.vder(s, t) }
                    #[inline(always)]
                    fn uuder(&self, s: f64, t: f64) -> Self::Vector { self.0.uuder(s, t) }
                    #[inline(always)]
                    fn uvder(&self, s: f64, t: f64) -> Self::Vector { self.0.uvder(s, t) }
                    #[inline(always)]
                    fn vvder(&self, s: f64, t: f64) -> Self::Vector { self.0.vvder(s, t) }
                    #[inline(always)]
                    fn der_mn(&self, m: usize, n: usize, s: f64, t: f64) -> Self::Vector { self.0.der_mn(m, n, s, t) }
                    #[inline(always)]
                    fn ders(&self, max_order: usize, s: f64, t: f64) { self.0.ders(max_order, s, t) }
                    #[inline(always)]
                    fn parameter_range(&self,) -> ((std::ops::Bound<f64>, std::ops::Bound<f64>), (std::ops::Bound<f64>, std::ops::Bound<f64>)) {
                        self.0.parameter_range()
                    }
                    #[inline(always)]
                    fn u_period(&self) -> Option<f64> { self.0.u_period() }
                    #[inline(always)]
                    fn v_period(&self) -> Option<f64> { self.0.v_period() }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ParametricSurface3D` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(ParametricSurface3D)]
pub fn derive_parametric_surface3d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name0 = quote! { truck_geotrait::ParametricSurface };
    let trait_name1 = quote! { truck_geotrait::ParametricSurface3D };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen
        .where_clause
        .iter()
        .flat_map(|x| &x.predicates)
        .collect::<Vec<_>>();
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods0 = methods!(
                variants,
                trait_name0,
                fn subs(&self, s: f64, t: f64) -> Self::Point,
                fn uder(&self, s: f64, t: f64) -> Self::Vector,
                fn vder(&self, s: f64, t: f64) -> Self::Vector,
                fn uuder(&self, s: f64, t: f64) -> Self::Vector,
                fn uvder(&self, s: f64, t: f64) -> Self::Vector,
                fn vvder(&self, s: f64, t: f64) -> Self::Vector,
                fn der_mn(&self, m: usize, n: usize, s: f64, t: f64) -> Self::Vector,
                fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector>,
                fn parameter_range(&self,) -> (truck_geotrait::ParameterRange, truck_geotrait::ParameterRange),
                fn u_period(&self,) -> Option<f64>,
                fn v_period(&self,) -> Option<f64>,
            );
            let methods1 = methods!(
                variants,
                trait_name1,
                fn normal(&self, u: f64, v: f64) -> Vector3,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name0 for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name0,)* {
                    type Point = Point3;
                    type Vector = Vector3;
                    #(#methods0)*
                }

                #[automatically_derived]
                impl #gen #trait_name1 for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name0,)* {
                    #(#methods1)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name0 for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name0, {
                    type Point = Point3;
                    type Vector = Vector3;
                    fn subs(&self, s: f64, t: f64) -> Self::Point { self.0.subs(s, t) }
                    fn uder(&self, s: f64, t: f64) -> Self::Vector { self.0.uder(s, t) }
                    fn vder(&self, s: f64, t: f64) -> Self::Vector { self.0.vder(s, t) }
                    fn uuder(&self, s: f64, t: f64) -> Self::Vector { self.0.uuder(s, t) }
                    fn uvder(&self, s: f64, t: f64) -> Self::Vector { self.0.uvder(s, t) }
                    fn vvder(&self, s: f64, t: f64) -> Self::Vector { self.0.vvder(s, t) }
                    fn der_mn(&self, m: usize, n: usize, s: f64, t: f64) -> Self::Vector { self.0.der_mn(m, n, u, v) }
                    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector> {
                        self.0.ders(max_order, u, v)
                    }
                    fn parameter_range(&self,) -> (truck_geotrait::ParmaterRange, truck_geotrait::ParameterRange) {
                        self.0.parameter_range()
                    }
                    fn u_period(&self) -> Option<f64> { self.0.u_period() }
                    fn v_period(&self) -> Option<f64> { self.0.v_period() }
                }
                #[automatically_derived]
                impl #gen #trait_name1 for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name0, {
                    fn normal(&self, u: f64, v: f64) -> Vector3 { self.0.normal(u, v) }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `SearchNearestParameter<D1>` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(SearchNearestParameterD1)]
pub fn derive_snp_d1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::SearchNearestParameter::<D1> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let methods = methods!(
                variants,
                trait_name,
                fn search_nearest_parameter<H: Into<SPHint1D>>(
                    &self,
                    pt: Self::Point,
                    hint: H,
                    trials: usize,
                ) -> Option<f64>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    type Point = <#field_type as #trait_name>::Point;
                    fn search_nearest_parameter<H: Into<SPHint1D>>(
                        &self,
                        pt: Self::Point,
                        hint: H,
                        trials: usize,
                    ) -> Option<f64> {
                        self.0.search_nearest_parameter(pt, hint, trials)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `SearchNearestParameter<D2>` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(SearchNearestParameterD2)]
pub fn derive_snp_d2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::SearchNearestParameter::<D2> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let methods = methods!(
                variants,
                trait_name,
                fn search_nearest_parameter<H: Into<SPHint2D>>(
                    &self,
                    pt: Self::Point,
                    hint: H,
                    trials: usize,
                ) -> Option<(f64, f64)>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    type Point = <#field_type as #trait_name>::Point;
                    fn search_nearest_parameter<H: Into<SPHint2D>>(
                        &self,
                        pt: Self::Point,
                        hint: H,
                        trials: usize,
                    ) -> Option<(f64, f64)> {
                        self.0.search_nearest_parameter(pt, hint, trials)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `SearchParameter<D1>` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(SearchParameterD1)]
pub fn derive_sp_d1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::SearchParameter::<D1> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let tys = &tys[1..];
            let methods = methods!(
                variants,
                trait_name,
                fn search_parameter<H: Into<SPHint1D>>(
                    &self,
                    pt: Self::Point,
                    hint: H,
                    trials: usize,
                ) -> Option<f64>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #top_ty: #trait_name,
                    #(#tys: #trait_name<Point = <#top_ty as #trait_name>::Point>,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    type Point = <#field_type as #trait_name>::Point;
                    fn search_parameter<H: Into<SPHint1D>>(
                        &self,
                        pt: Self::Point,
                        hint: H,
                        trials: usize,
                    ) -> Option<f64> {
                        self.0.search_parameter(pt, hint, trials)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `SearchParameter<D2>` for Enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(SearchParameterD2)]
pub fn derive_sp_d2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::SearchParameter::<D2> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let top_ty = &tys[0];
            let methods = methods!(
                variants,
                trait_name,
                fn search_parameter<H: Into<SPHint2D>>(
                    &self,
                    pt: Self::Point,
                    hint: H,
                    trials: usize,
                ) -> Option<(f64, f64)>,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    type Point = <#field_type as #trait_name>::Point;
                    fn search_parameter<H: Into<SPHint2D>>(
                        &self,
                        pt: Self::Point,
                        hint: H,
                        trials: usize,
                    ) -> Option<(f64, f64)> {
                        self.0.search_parameter(pt, hint, trials)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `Transformed<Matrix3>` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(TransformedM3)]
pub fn derive_transform_m3(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::Transformed::<Matrix3> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn transform_by(&mut self, mat: truck_geometry::prelude::Matrix3) -> (),
                fn transformed(&self, mat: truck_geometry::prelude::Matrix3) -> Self,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn transform_by(&mut self, mat: truck_geometry::prelude::Matrix3) {
                        self.0.transformed(mat)
                    }
                    fn transformed(&self, mat: truck_geometry::prelude::Matrix3) -> Self {
                        Self(self.0.transformed(mat))
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `Transformed<Matrix4>` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(TransformedM4)]
pub fn derive_transform_m4(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_geotrait::Transformed::<Matrix4> };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn transform_by(&mut self, mat: truck_geometry::prelude::Matrix4) -> (),
                fn transformed(&self, mat: truck_geometry::prelude::Matrix4) -> Self,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn transform_by(&mut self, mat: truck_geometry::prelude::Matrix4) {
                        self.0.transformed(mat)
                    }
                    fn transformed(&self, mat: truck_geometry::prelude::Matrix4) -> Self {
                        Self(self.0.transformed(mat))
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}
/// Derive macro generating an impl of the trait `StepLength` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(StepLength)]
pub fn derive_step_length(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_stepio::out::StepLength };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn step_length(&self,) -> usize,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn step_length(&self) -> usize { self.0.step_length() }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `DisplayByStep` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(DisplayByStep)]
pub fn derive_display_by_step(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_stepio::out::DisplayByStep };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        #trait_name::fmt(&self.0, idx, f)
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `StepCurve` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(StepCurve)]
pub fn derive_step_curve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_stepio::out::StepCurve };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn same_sense(&self,) -> bool,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn same_sense(&self) -> bool { #trait_name::same_sense(&self.0) }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `StepSurface` for enums or single field tuple structs.
#[proc_macro_error]
#[proc_macro_derive(StepSurface)]
pub fn derive_step_surface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { truck_stepio::out::StepSurface };
    let ty = input.ident;
    let gen = input.generics;
    let where_predicates = gen.where_clause.iter().flat_map(|x| &x.predicates);
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let variant = variants.into_iter().next().expect("empty enum!");
            let tys: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
            let methods = methods!(
                variants,
                trait_name,
                fn same_sense(&self,) -> bool,
            );
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #(#tys: #trait_name,)* {
                    #(#methods)*
                }
            }
        }
        Data::Struct(DataStruct { ref fields, .. }) => {
            let field: Vec<_> = fields.iter().collect();
            if field.len() != 1 || field[0].ident.is_some() {
                unimplemented!();
            }
            let field_type = &field[0].ty;
            quote! {
                #[automatically_derived]
                impl #gen #trait_name for #ty #gen
                where
                    #(#where_predicates,)*
                    #field_type: #trait_name, {
                    fn same_sense(&self) -> bool { #trait_name::same_sense(&self.0) }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

/// Derive macro generating an impl of the trait `ToSameGeometry<Self>` for structs.
#[proc_macro_error]
#[proc_macro_derive(SelfSameGeometry)]
pub fn derive_self_same_geometry(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let gen = input.generics;
    quote! {
        #[automatically_derived]
        impl #gen ToSameGeometry<Self> for #ident #gen where Self: Clone {
            #[inline(always)]
            fn to_same_geometry(&self) -> Self { self.clone() }
        }
    }
    .into()
}
