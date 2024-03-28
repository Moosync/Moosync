import { Server, ServiceDefinition } from "@grpc/grpc-js";

export interface ExtensionAPIServer extends Server {
	GetExtensions(): unknown;
}

export interface ExtensionAPIGRPC {
	extensions: {
		ExtensionAPI: {
			service: ServiceDefinition<ExtensionAPIServer>;
		};
	};
}
