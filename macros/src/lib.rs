use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    FnArg, ItemFn, ReturnType, parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma,
};

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // 默认配置
    let mut add_state = true;
    let mut add_connect_info = false;

    // 解析属性参数
    if !attr.is_empty() {
        let attrs: Punctuated<syn::Meta, Comma> =
            parse_macro_input!(attr with Punctuated::parse_terminated);

        for attr in attrs {
            match attr {
                syn::Meta::NameValue(nv) if nv.path.is_ident("state") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(b),
                        ..
                    }) = nv.value
                    {
                        add_state = b.value;
                    }
                }
                syn::Meta::NameValue(nv) if nv.path.is_ident("connect_info") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(b),
                        ..
                    }) = nv.value
                    {
                        add_connect_info = b.value;
                    }
                }
                _ => {}
            }
        }
    }

    let ItemFn {
        attrs: fn_attrs,
        vis,
        sig,
        block,
    } = input_fn;

    // 处理返回类型
    let return_type = match &sig.output {
        ReturnType::Default => {
            quote! { -> crate::error::ApiResult<crate::router::ApiResponse<()>> }
        }
        ReturnType::Type(_, ty) => {
            let ty_str = ty.to_token_stream().to_string();
            if ty_str.contains("ApiResult") {
                quote! { -> #ty }
            } else {
                quote! { -> crate::error::ApiResult<crate::router::ApiResponse<#ty>> }
            }
        }
    };

    // 检查是否已存在State和ConnectInfo参数
    let mut has_state = false;
    let mut has_connect_info = false;

    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let ty_str = pat_type.ty.to_token_stream().to_string();
            if ty_str.contains("State") {
                has_state = true;
            } else if ty_str.contains("ConnectInfo") {
                has_connect_info = true;
            }
        }
    }

    // 构建新参数列表
    let mut new_inputs = Punctuated::new();

    // 添加State参数（如果配置为true且尚未存在）
    if add_state && !has_state {
        let state_arg: FnArg = parse_quote! {
            axum::extract::State(crate::app::AppState { db }): axum::extract::State<crate::app::AppState>
        };
        new_inputs.push(state_arg);
    }

    // 添加ConnectInfo参数（如果配置为true且尚未存在）
    if add_connect_info && !has_connect_info {
        let connect_info_arg: FnArg = parse_quote! {
            axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>
        };
        new_inputs.push(connect_info_arg);
    }

    // 添加原始参数
    for input in &sig.inputs {
        new_inputs.push(input.clone());
    }

    // 构建新函数签名
    let mut new_sig = sig.clone();
    new_sig.inputs = new_inputs;
    new_sig.output = parse_quote!(#return_type);

    // 构建新函数
    let expanded = quote! {
        #(#fn_attrs)*
        #vis #new_sig {
            #block
        }
    };

    TokenStream::from(expanded)
}
