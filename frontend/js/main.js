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
        return e("li", {className: "ListItem"},
                 e("span", {className: "ItemName"},
                   e("a", {"href": this.state.url, className: "ItemLink"},
                     this.state.name)),
                 e("span", {className: "ItemPrice"}, this.state.price),
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
        return e("ul", {id: "Wishlist"}, items);
    }
}

ReactDOM.render(e(WishList, null), document.getElementById('WishlistWrapper'));
console.log("Loaded.");
