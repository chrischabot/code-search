
function greet(name) {
    console.log(`Hello, ${name}!`);
}

const useState = (initial) => {
    let state = initial;
    return [state, (newState) => { state = newState; }];
};

class Component {
    constructor() {
        this.state = {};
    }
    
    render() {
        return null;
    }
}
