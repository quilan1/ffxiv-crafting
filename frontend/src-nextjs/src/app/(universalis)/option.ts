type FnOpt<T, U> = (_: T) => OptionType<U>;
type Fn<T, U> = (_: T) => U;

export class OptionType<T> {
    private _is_none: boolean;
    private value?: T;

    constructor(is_none: boolean, value?: T) {
        this._is_none = is_none;
        this.value = value;
    }

    is_some(): boolean {
        return !this._is_none;
    }

    is_none(): boolean {
        return this._is_none;
    }

    ////////

    map<U>(f: FnOpt<T, U>): OptionType<U> {
        if (this._is_none) return None();
        return f(this.value as T);
    }

    map_or<U>(d: U, f: Fn<T, U>): OptionType<U> {
        if (this._is_none) return Some(d);
        return Some(f(this.value as T));
    }

    and<U>(v: OptionType<U>): OptionType<U> {
        if (this._is_none) return None();
        return v;
    }

    and_then<U>(f: FnOpt<T, U>): OptionType<U> {
        if (this._is_none) return None();
        return f(this.value as T);
    }

    flatmap<U>(f: FnOpt<T, U>): OptionType<U> {
        if (this._is_none) return None();
        return f(this.value as T);
    }

    or(v: OptionType<T>): OptionType<T> {
        if (this._is_none) return v;
        return Some(this.value as T);
    }

    zip<U>(v: OptionType<U>): OptionType<[T, U]> {
        if (this._is_none) return None();
        if (v._is_none) return None();
        return Some([this.value as T, v.unwrap()]);
    }

    zip_all<U>(...params: OptionType<U>[]): OptionType<(U|T)[]> {
        if (this._is_none) return None();
        if (params.some(v => v.is_none())) return None();
        const values = params.map(v => v.unwrap());
        return Some([this.value!, ...values]);
    }

    ////////

    expect(msg: string): T {
        if (this._is_none) throw new Error(msg);
        return this.value!;
    }

    unwrap(): T {
        return this.expect("Attempting to unwrap a None value");
    }

    unwrap_or(v: T): T {
        if (this._is_none) return v;
        return this.value!;
    }
}

export const Some = <T>(v: T) => {
    return new OptionType(false, v);
}

export const None = <T=never>() => {
    return new OptionType<T>(true);
}
