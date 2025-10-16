use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, ItemFn, FnArg, ReturnType,
    punctuated::Punctuated, token::Comma, parse_quote
};

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // 默认配置
    let mut add_state = true;

    // 解析属性参数（只处理state配置）
    if !attr.is_empty() {
        let attrs: Punctuated<syn::Meta, Comma> = parse_macro_input!(attr with Punctuated::parse_terminated);
        
        for attr in attrs {
            match attr {
                syn::Meta::NameValue(nv) if nv.path.is_ident("state") => {
                    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = nv.value {
                        add_state = b.value;
                    }
                },
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
        ReturnType::Default => quote! { -> ApiResult<ApiResponse<()>> },
        ReturnType::Type(_, ty) => {
            let ty_str = ty.to_token_stream().to_string();
            if ty_str.contains("ApiResult") {
                quote! { -> #ty }
            } else {
                quote! { -> ApiResult<ApiResponse<#ty>> }
            }
        }
    };

    // 检查是否已存在State参数
    let has_state = sig.inputs.iter().any(|input| {
        if let FnArg::Typed(pat_type) = input {
            let ty_str = pat_type.ty.to_token_stream().to_string();
            ty_str.contains("State")
        } else {
            false
        }
    });
    
    // 构建新参数列表
    let mut new_inputs = Punctuated::new();
    
    // 添加State参数到第一个位置（如果配置为true且尚未存在）
    if add_state && !has_state {
        let state_arg: FnArg = parse_quote! {
            State(AppState { db }): State<AppState>
        };
        new_inputs.push(state_arg);
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