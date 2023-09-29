export type ApiInfo = {
    endpoint: string,
    options?: object,
}

export type ApiCall = {
    get?: ApiInfo,
    put?: ApiInfo,
}

export default class Api {
    private static readonly WEBPAGE: string = "127.0.0.1:3001";

    static getUrl(path: string): string {
        return `${this.WEBPAGE}/${path}`;
    }

    private static fetch(path: string, args?: object): Promise<Response> {
        return window.fetch(`http://${this.WEBPAGE}/${path}`, args);
    }

    static async call<T>(call: ApiInfo, formData?: object, body?: object): Promise<T> {
        const arr = [];
        if (formData !== undefined) {
            for (const [key, value] of Object.entries(formData)) {
                arr.push(`${key}=${window.encodeURIComponent(value)}`);
            }
        }
        let formDataStr = arr.join('&');

        let options = call.options as any;
        if (body !== undefined) {
            options.body = JSON.stringify(body);
        }

        try {
            let request = await this.fetch(`${call.endpoint}?${formDataStr}`, options);
            let text = await request.text();
            try {
                return await JSON.parse(text);
            } catch (err) {
                return text as any;
            }
        } catch (err) {
            console.error(err);
            throw err;
        }
    }
}