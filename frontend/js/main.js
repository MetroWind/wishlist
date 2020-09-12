const e = React.createElement;

class WishListItem extends React.Component
{
    constructor(props)
    {
        super(props);
        this.state = { name: "", url: "", price: "" };
    }

    componentDidMount()
    {
        const url = `api/get?store=${encodeURIComponent(this.props.store)}&\
id=${encodeURIComponent(this.props.id)}`;
        console.log(url);
        fetch(url).then(res => res.json())
            .then(data => this.setState(
                { name: data.name,
                  url: data.url,
                  price: data.price_str }));
    }

    render()
    {
        return e("li", null,
                 e("span", null,
                   e("a", {"href": this.state.url}, this.state.name)),
                 e("span", null, this.state.price),
                );
    }
}


class WishList extends React.Component
{
    constructor(props)
    {
        super(props);
        this.state = { items: [] };
    }

    componentDidMount()
    {
        fetch("api/list").then(res => res.json())
            .then(data => this.setState({ items: data }));
    }

    render()
    {
        let items = this.state.items.map((s) => e(WishListItem, {store: s.store, id: s.id, key: s.id}));
        return e("ul", null, items);
    }
}

ReactDOM.render(e(WishList, null), document.getElementById('Wishlist'));
console.log("Loaded.");
