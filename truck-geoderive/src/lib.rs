//! Define some derive macros for implements geotraits.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::*;
use syn::*;

fn top_type_of_enumeration<'a>(
    variants: impl IntoIterator<Item = &'a Variant> + 'a,
) -> TokenStream2 {
    let variant = variants.into_iter().next().expect("empty enum!");
    let vec: Vec<_> = variant.fields.iter().collect();
    match vec.len() {
        0 => panic!("empty field!"),
        1 => vec[0].ty.to_token_stream(),
        _ => unimplemented!(),
    }
}

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
        $(fn $name: ident <$($gen: ident: $path: path),*> (
            $self_field: expr,
            $($var: ident: $ty: ty),*$(,)?
        ) -> $return_type: ty),*$(,)?
    ) => {
        vec![$(Method {
            name: quote! { $name },
            generics: Some(quote! { <$($gen: $path),*> }),
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
        let vals = self
            .fields
            .iter()
            .map(|f| f.var.to_token_stream())
            .collect::<Vec<_>>();
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
            #[inline(always)]
            fn #method_name #generics (#self_field #(#fields),*) -> #return_type { #implement }
        }
    }
}

#[proc_macro_error]
#[proc_macro_derive(BoundedCurve)]
pub fn derive_bounded_curve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { BoundedCurve };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods! {
                variants, trait_name,
                fn parameter_range(&self,) -> (f64, f64),
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(BoundedSurface)]
pub fn derive_bounded_surface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { BoundedSurface };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods! {
                variants, trait_name,
                fn parameter_range(&self,) -> ((f64, f64), (f64, f64)),
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(Cut)]
pub fn derive_cut(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { Cut };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods! {
                variants, trait_name,
                fn cut(&mut self, t: f64) -> Self,
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(Invertible)]
pub fn derive_invertible(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { Invertible };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods! {
                variants, trait_name,
                fn invert(&mut self,) -> (),
                fn inverse(&self,) -> Self,
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(ParameterDivision1D)]
pub fn derive_parameter_division_1d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { ParameterDivision1D };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
            let methods = methods! {
                variants, trait_name,
                fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>),
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(ParameterDivision2D)]
pub fn derive_parameter_division_2d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { ParameterDivision2D };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods! {
                variants, trait_name,
                fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>),
            };
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(ParametricCurve)]
pub fn derive_parametric_curve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { ParametricCurve };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
            let methods = methods!(
                variants,
                trait_name,
                fn subs(&self, t: f64) -> Self::Point,
                fn der(&self, t: f64) -> Self::Vector,
                fn der2(&self, t: f64) -> Self::Vector,
            );
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    type Vector = <#top_ty as #trait_name>::Vector;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(ParametricSurface)]
pub fn derive_parametric_surface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { ParametricSurface };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
            let methods = methods!(
                variants,
                trait_name,
                fn subs(&self, s: f64, t: f64) -> Self::Point,
                fn uder(&self, s: f64, t: f64) -> Self::Vector,
                fn vder(&self, s: f64, t: f64) -> Self::Vector,
                fn uuder(&self, s: f64, t: f64) -> Self::Vector,
                fn uvder(&self, s: f64, t: f64) -> Self::Vector,
                fn vvder(&self, s: f64, t: f64) -> Self::Vector,
            );
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    type Vector = <#top_ty as #trait_name>::Vector;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(ParametricSurface3D)]
pub fn derive_parametric_surface3d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { ParametricSurface3D };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let methods = methods!(
                variants,
                trait_name,
                fn normal(&self, u: f64, v: f64) -> Vector3,
            );
            quote! {
                #[automatically_derived]
                impl #gen truck_geotrait::#trait_name for #ty {
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}


#[proc_macro_error]
#[proc_macro_derive(SearchNearestParameterD1)]
pub fn derive_snp_d1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { SearchNearestParameter::<D1> };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
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
                impl #gen truck_geotrait::SearchNearestParameter<D1> for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(SearchNearestParameterD2)]
pub fn derive_snp_d2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { SearchNearestParameter::<D2> };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
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
                impl #gen truck_geotrait::SearchNearestParameter<D2> for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(SearchParameterD1)]
pub fn derive_sp_d1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { SearchParameter::<D1> };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
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
                impl #gen truck_geotrait::SearchParameter<D1> for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(SearchParameterD2)]
pub fn derive_sp_d2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let trait_name = quote! { SearchParameter::<D2> };
    let ty = input.ident;
    let gen = input.generics;
    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let top_ty = top_type_of_enumeration(variants);
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
                impl #gen truck_geotrait::SearchParameter<D2> for #ty {
                    type Point = <#top_ty as #trait_name>::Point;
                    #(#methods)*
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}
