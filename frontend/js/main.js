const e = React.createElement;

class WishListItem extends React.Component
{
    constructor(props)
    {
        super(props);
        this.state = { show_history: false };
        this.price_history;
        this.onPriceLink = this.onPriceLink.bind(this);
    }

    queryHistory()
    {
        fetch(`api/price_history/${this.props.store}/${this.props.in_store_id}`)
            .then(res => res.json()).then(data => {
                console.log(data);
                this.price_history = data;
                this.setState({show_history: true});
            });
    }

    onPriceLink(_)
    {
        if(this.state.show_history == true)
        {
            this.setState({show_history: false});
        }
        else
        {
            this.queryHistory();
        }
    }

    componentDidUpdate()
    {
        if(this.state.show_history == true)
        {
            this.drawPriceChart();
        }
    }

    drawPriceChart()
    {
        let Wrapper = document.getElementById(`PriceChart-${this.props.store}-${this.props.in_store_id}`);
        const Margin = {top: 20, right: 15, bottom: 25, left: 32};
        const Height = 150;
        const Width = Wrapper.clientWidth;
        const WidthInner = Width - Margin.left - Margin.right;
        const HeightInner = Height - Margin.top - Margin.bottom;

        const Data = this.price_history;
        let Canvas = d3.select(`#PriceChart-${this.props.store}-${this.props.in_store_id}`)
            .append("svg").attr("width",  Width).attr("height", Height);

        let Plot = Canvas.append("g")
            .attr("transform", `translate(${Margin.left}, ${Margin.top})`);
        let Line = d3.line()
            .x(pp => new Date(pp.time*1000))
            .y(pp => pp.price)
            .curve(d3.curveStepAfter);
        Plot.append("path").attr("class", "PriceLine")
            .attr("d", (d,i) => Line(Data)
            );
        var ScaleX = d3.scaleTime().range([0, WidthInner])
            .domain(d3.extent(Data, pp => new Date(pp.time*1000)));
        var ScaleY = d3.scaleLinear().range([HeightInner, 0])
            .domain([0, d3.max(Data, pp => pp.price)]);

        var AxisX = d3.axisBottom(ScaleX);
        var AxisY = d3.axisLeft(ScaleY);
        Plot.append("g").attr("transform", `translate(0, ${HeightInner})`).call(AxisX);
        Plot.append("g").call(AxisY);

        // Plotly.newPlot(this.price_chart.current, [{
        //     x: Data.map(pp => pp.time),
        //     y: Data.map(pp => pp.price),
        //     line: {shape: "hv"},
        //     color: "transparent",
        // }], {
        //     showlegend: false,
        // }, {staticPlot: true});
    }

    render()
    {
        let item_info =
            e("div", {className: "ItemInfo"},
              e("span", {className: "ItemName"},
                e("a", {"href": this.props.url, className: "ItemLink"},
                  this.props.name)),
              e("span", {className: "ItemPrice"},
                e("a", {"href": "#", className: "PriceLink", onClick: this.onPriceLink },
                  this.props.price)));

        if(this.state.show_history == false)
        {
            return e("li", {className: "ListItem"}, item_info);
        }
        else
        {
            let price_chart = e("div", {className: "PriceChart", id: `PriceChart-${this.props.store}-${this.props.in_store_id}`});
            return e("li", {className: "ListItem"}, [item_info, price_chart]);
        }
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
        let items = this.state.items.map((s) =>
            e(WishListItem, {url: s.url, name: s.name, price: s.price_str,
                             store: s.store, in_store_id: s.id}));
        return e("ul", {id: "Wishlist"}, items);
    }
}

ReactDOM.render(e(WishList, null), document.getElementById('WishlistWrapper'));
