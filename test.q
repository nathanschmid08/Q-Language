// test.q

system.init{
    "type": variable,
    "name": my_var,
    "datatype": string,
    "value": "Hello from Q!"
};

system.log{
    "type": info,
    arguments{
        my_var.value
    },
    "message": "The value of my_var is: " & my_var.value
};

function my_func(p1 in number) {
    system.log{
        "type": info,
        arguments{
            p1.value
        },
        "message": "my_func was called with: " & p1.value
    };
    return null;
};

system.exec{
    "type": function,
    "name": my_func,
    parameters{
        p1 => 42
    }
};

system.set{
    "name": my_var,
    "value": "Q is awesome!"
};

system.log{
    "type": info,
    arguments{
        my_var.value
    },
    "message": "The new value of my_var is: " & my_var.value
};
