use yew::prelude::*;



#[function_component]
fn List() -> Html {
    html! {
        <ul>
            <li>{1}</li>
            <li>{2}</li>
        </ul>
    }
}
#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <List/>
            <h1 class="text-black text-xl">{"Hello world"}</h1>
        </div>
    }
}
fn main() {
    yew::Renderer::<App>::new().render();
}
