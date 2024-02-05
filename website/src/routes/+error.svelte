<div class="container">
    <div class="row row--grid">
        <!-- error -->
        <div class="col-12">
            <div class="page-404">
                <div class="page-404__wrap">
                    <div class="page-404__content">
                        <h1 class="page-404__title">404</h1>
                        <p class="page-404__text">Nothing here !</p>
                        <a href="/" class="page-404__btn">go back</a>
                    </div>
                </div>
            </div>
        </div>
        <!-- end error -->
    </div>
</div>

<style>
    .page-404 {
        display: block;
        position: relative;
    }

    .page-404__wrap {
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        width: 100%;
        height: auto;
        padding: 25px 0 0;
    }

    .page-404__content {
        background-color: #16151a;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        border-radius: 12px;
        padding: 20px;
        position: relative;
        width: 100%;
        max-width: 420px;
        border: 1px solid #222227;
    }

    .page-404__title {
        position: relative;
        color: #fff;
        line-height: 100%;
        font-size: 100px;
        margin-bottom: 10px;
        font-weight: 500;
        font-family: 'Inter', sans-serif;
    }

    .page-404__text {
        text-align: center;
        display: block;
        width: 100%;
        color: #bdbdbd;
        font-size: 16px;
        line-height: 24px;
        margin-bottom: 30px;
    }

    .page-404__btn {
        width: 100%;
        height: 50px;
        border-radius: 12px;
        background-color: #AC1E21;
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        color: #fff;
        font-size: 14px;
        position: relative;
        z-index: 2;
        text-transform: uppercase;
        font-family: 'Inter', sans-serif;
        font-weight: 500;
        margin: 0 auto;
    }

    .page-404__btn:hover {
        color: #fff;
        background-color: #222227;
    }

    @media (min-width: 576px) {
        .page-404__content {
            padding: 40px;
        }
    }

</style>