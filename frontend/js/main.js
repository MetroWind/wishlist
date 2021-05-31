const e = React.createElement;

class WishListItem extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        return e("li", {className: "ListItem"},
                 e("span", {className: "ItemName"},
                   e("a", {"href": this.props.url, className: "ItemLink"},
                     this.props.name)),
                 e("span", {className: "ItemPrice"}, this.props.price),
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
        let items = this.state.items.map((s) => e(WishListItem, {url: s.url, name: s.name, price: s.price_str}));
        return e("ul", {id: "Wishlist"}, items);
    }
}

ReactDOM.render(e(WishList, null), document.getElementById('WishlistWrapper'));
