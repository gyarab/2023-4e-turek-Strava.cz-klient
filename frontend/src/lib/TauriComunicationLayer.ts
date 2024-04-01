import { invoke } from '@tauri-apps/api/tauri';
import { goto } from '$app/navigation';
import { json } from '@sveltejs/kit';

const login = async (username: string, value: string, cantine: number) : Promise<Result<User,string>> =>{
    let res = await invoke('login', {
        username: username,
        password: value,
        cantine: cantine,
    })
    return res as Result<User,string>; 
}
const getUserMenu = async (): Promise<Result<Menu, string>> => {
    let res = await invoke('get_menu_data') as Result<[string[],Menu], string>;
    console.log(res);
    switch(res._t){
        case 'success':
            let data =  new Map<string,DailyMenu>();
            for (let i =0; i< res.data[0].length; i++){
                data.set(res.data[0][i],res.data[1][res.data[0][i]]);
            }
            console.log(data);
            return {_t: 'success', data: Object.fromEntries(data)};
        case 'failure':
            return {_t: 'failure', error: res.error};
        case 'unauthorized':
            return {_t: 'unauthorized'};
    }
};
const orderDish = async (dishId: string, status: boolean): Promise<Result<number, string>> => {
    return await invoke('order_dish', {
        dishId: dishId,
        status: status,
    }) as Result<number,string>;
};
const saveOrder = async (): Promise<Result<string, SaveFailureResponse>> => {
	return await invoke('save_orders') as Result<string, SaveFailureResponse>;
};
const logout = async (): Promise<void> => {
	await invoke('logout');
};
const queryCantineHistory = async (
	cantineId: string,
	query: string,
	list: string
): Promise<Result<Dish[], string>> => {
	 let n = await invoke('query_cantine_history', {cantineId: cantineId, query: query, listToQuery: list}) as Result<Dish[], string>;
	 console.log(n);
	 return n;
};

const querySettings = async (query: string, listToQuery: string): Promise<Result<Dish[], string>> => {
	return await invoke('query_settings', {query: query, list_to_query: listToQuery}) as Result<Dish[], string>;
};
const fetchSettings = async (): Promise<Result<Settings, string>> => {
  let n = await invoke('fetch_settings') as Result<Settings, string>;
  console.log(n);
  return n;
};
const updateSettings = async (settingsItem: string|Dish, action: string, list:string): Promise<Result<string, string>> => {
	return await invoke('update_settings', {settings_item: settingsItem, action: action, list: list}) as Result<string, string>;
};
export {
	login,
	getUserMenu,
	orderDish,
	saveOrder,
	logout,
	queryCantineHistory,
	querySettings,
	fetchSettings,
	updateSettings
};