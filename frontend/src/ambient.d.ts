type DishInfo = {
	id: string;
	allergens: string[];
	orderState: boolean;
};
type Dish = {
	name: string;
	allergens: string[];
}
type OrderDishRequest = {
	id: string;
	status: boolean;
};
type OrderDishResponse = {
	account: number;
};
type SaveFailureResponse = {
	message: string;
	account: number;
};
type LoginResponse = {
	message: string;
	user: User;
};
type User = {
	username: string;
	account: number;
};

type MenuResponse = {
	menu: Menu;
};
type ErrorResponse = {
	message: string;
};
type SuccessResponse = {
	message: string;
};
type Success<T> = {
	_t: 'success';
	data: T;
};
type Failure<T> = {
	_t: 'failure';
	error: T;
};
type Unauthorized = {
	_t: 'unauthorized';
};
type Result<T, R> = Success<T> | Failure<R>| Unauthorized;
type Menu = {
	[key: string]: DailyMenu;
};
type DailyMenu = {
	[key: string]: DishInfo;
};
type MenuData = { [key: string]: DailyMenu };
type Settings = {
	blacklistedDishes: Dish[];
	blacklistedAllergens: string[];
	whitelistedDishes: Dish[];
	strategy: string;
}
type QueryResponse<T> = {
   result: T[];
}
type strategy = {
	id: string;
	name: string;
	value: string;
}
declare module 'TauriComunicationLayer' {
	export function login(
		username: string,
		password: string,
		cantine: number,
		stayLogged: boolean
	): Promise<string>;
}
declare module 'WebComunicationLayer' {
	export function login(
		username: string,
		password: string,
		cantine: number,
		stayLogged: boolean
	): Promise<Result>;
	export function getUserMenu(): Promise<MenuData>;
	export function orderDish(dishId: string, status: boolean): Promise<Result<void, string>>;
}
