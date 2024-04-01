const sendRequest = async <S, F, T, R>(
	path: string,
	method: string,
	body: any,
	failureAttribute: string,
	successAttribute: string
): Promise<Result<T, R>> => {
	let request: any = {
		method: method,
		credentials: 'include',
		headers: {
			// 'csrf-token': 'nocheck',
			'Content-Type': 'application/json;charset=UTF-8'
		}
	};
	let url: string = `https://strava.jumpingcrab.com${path}`;
	if (method === 'POST') {
		request.body = JSON.stringify(body);
	}
	let res = await fetch(url, request);
	console.log(res);
	let data = await res.json();
	if (res.status === 401) {
		return { _t: 'unauthorized' };
	}
	if (res.status === 200) {
		if (successAttribute === '') {
			return { _t: 'success', data: data as T };
		}
		return { _t: 'success', data: data[successAttribute as keyof S] };
	}
	if (failureAttribute === '') {
		return { _t: 'failure', error: data as R };
	}
	return { _t: 'failure', error: data[failureAttribute as keyof F] };
};

const login = async (
	username: string,
	value: string,
	cantine: number,
): Promise<Result<User, string>> => {
	const user = {
		jmeno: username,
		heslo: value,
		cislo: cantine,
		zustatPrihlasen: false,
		lang: 'CZ'
	};
	return await sendRequest<LoginResponse, ErrorResponse, User, string>(
		'/login',
		'POST',
		user,
		'message',
		'user'
	);
};
const getUserMenu = async (): Promise<Result<Menu, string>> => {
	return await sendRequest<MenuResponse, ErrorResponse, Menu, string>(
		'/user_menu',
		'GET',
		null,
		'message',
		'menu'
	);
};
const orderDish = async (dishId: string, status: boolean): Promise<Result<number, string>> => {
	return await sendRequest<OrderDishResponse, ErrorResponse, number, string>(
		'/order_dish',
		'POST',
		{ id: dishId, status: status },
		'message',
		'account'
	);
};
const saveOrder = async (): Promise<Result<string, SaveFailureResponse>> => {
	return await sendRequest<SuccessResponse, SaveFailureResponse, string, SaveFailureResponse>(
		'/save_orders',
		'POST',
		null,
		'',
		'message'
	);
};
const logout = async (): Promise<void> => {
	await sendRequest<SuccessResponse, ErrorResponse, string, string>(
		'/logout',
		'POST',
		null,
		'message',
		'message'
	);
};
const queryCantineHistory = async (
	cantineId: string,
	query: string,
	list: string
): Promise<Result<Dish[], string>> => {
	let url = `/cantine_history?cantine_id=${encodeURIComponent(
		cantineId
	)}&query=${encodeURIComponent(query)}&list=${encodeURIComponent(list)}`;
	return await sendRequest<QueryResponse<Dish>, ErrorResponse, Dish[], string>(
		url,
		'GET',
		null,
		'message',
		'result'
	);
};

const querySettings = async (query: string, listToQuery: string): Promise<Result<Dish[], string>> => {
	let url = `/settings_query?query=${encodeURIComponent(query)}&list=${encodeURIComponent(listToQuery)}`;
	return await sendRequest<QueryResponse<string>, ErrorResponse, Dish[], string>(
		url,
		'GET',
		null,
		'message',
		'result'
	);
};
const fetchSettings = async (): Promise<Result<Settings, string>> => {
	type SettingsResponse = {
		settings: Settings ;
	};
	return await sendRequest<SettingsResponse, ErrorResponse, Settings, string>(
		'/user_settings',
		'GET',
		null,
		'message',
		'settings'
	);
};
const updateSettings = async (settingsItem: string|Dish, action: string, list:string): Promise<Result<string, string>> => {
	return await sendRequest<SuccessResponse, ErrorResponse, string, string>(
		`/user_settings?action=${encodeURIComponent(action)}&list=${list}`,
		'POST',
		settingsItem,
		'message',
		'message'
	);
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
