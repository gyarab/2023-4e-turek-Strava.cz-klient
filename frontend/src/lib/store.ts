import { writable } from "svelte/store";

const amount = localStorage.getItem("account") || "0";
const account = writable(amount);
account.subscribe(value => {
    localStorage.setItem("account", value);
}); 

const code = localStorage.getItem("cantine") || "0000";
const cantine = writable(code);
cantine.subscribe(value => {
    localStorage.setItem("cantine", value);
}); 
const  user = localStorage.getItem("username") || "Nepřihlášen";
const username = writable(user);
username.subscribe(value => {
    localStorage.setItem("username", value);
}); 

export {account, cantine, username}