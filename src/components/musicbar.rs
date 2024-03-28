use leptos::{component, view, IntoView};

#[component]
fn Details() -> impl IntoView {
    view! {
        <div class="row no-gutters align-items-center w-100">
            <div class="col-auto">
                <img
                    class="img-fluid coverimg"
                    referrerpolicy="no-referrer"
                    src=""
                    alt="cover art"
                />
            </div>
            <div class="col text-truncate">
                <div class="row align-items-center justify-content-start">
                    <div class="col-auto w-100 d-flex">
                        <div title="hello" class="text song-title text-truncate mr-2">
                            Hello
                        </div>
                    </div>
                </div>

                <div class="row no-gutters">
                    <div class="col d-flex">
                        <div title="artist-name" class="text song-subtitle text-truncate">
                            Artist name
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MusicBar() -> impl IntoView {
    view! {
        <div class="musicbar-content d-flex">
            <div class="background w-100">
                <div class="musicbar h-100">

                    <div class="container-fluid d-flex bar-container h-100 pb-2">
                        <div class="row no-gutters align-items-center justify-content-center align-content-center no-gutters w-100 control-row justify-content-between">
                            <div class="col-4 no-gutters details-col w-100">
                                <Details/>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}
