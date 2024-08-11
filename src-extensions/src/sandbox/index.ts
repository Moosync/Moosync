/*
 *  index.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import type { mainRequests, providerFetchRequests } from "../constants";
import { mainRequestsKeys, providerExtensionKeys } from "../constants";
import log from "loglevel";

import { ExtensionHandler } from "../sandbox/extensionHandler";
import type EventEmitter from "node:events";

export class ExtensionHostIPCHandler {
  private extensionHandler: ExtensionHandler;
  private mainRequestHandler: MainRequestHandler; // private logsPath: string;

  constructor(bus: EventEmitter) {
    let extensionPath = "";
    let logsPath = "";
    let installPath = "";
    for (const [index, arg] of process.argv.entries()) {
      if (process.argv[index + 1]) {
        if (arg === "-extensionPath") {
          extensionPath = process.argv[index + 1];
        }

        if (arg === "-logPath") {
          logsPath = process.argv[index + 1];
        }

        if (arg === "-installPath") {
          installPath = process.argv[index + 1];
        }
      }
    }

    console.log("extension paths", extensionPath, logsPath, installPath);

    // this.logsPath = logsPath;
    this.setupLogger();

    this.extensionHandler = new ExtensionHandler(
      [extensionPath],
      logsPath,
      installPath,
      bus,
    );
    this.mainRequestHandler = new MainRequestHandler(this.extensionHandler);

    this.registerListeners();
    this.extensionHandler.startAll();
  }

  private setupLogger() {
    const logger = log.getLogger("Extension Host");
    // prefixLogger(this.logsPath, logger);
    const logLevel = process.env.DEBUG_LOGGING
      ? log.levels.DEBUG
      : log.levels.INFO;
    logger.setLevel(logLevel);

    console.info = (...args: unknown[]) => {
      logger.info(...args);
    };

    console.error = (...args: unknown[]) => {
      logger.error(...args);
    };

    console.warn = (...args: unknown[]) => {
      logger.warn(...args);
    };

    console.debug = (...args: unknown[]) => {
      logger.debug(...args);
    };

    console.trace = (...args: unknown[]) => {
      logger.trace(...args);
    };
  }

  private isExtensionEvent(key: keyof MoosyncExtensionTemplate) {
    return key === "onStarted" || key === "onStopped";
  }

  private isMainRequest(key: string) {
    return mainRequestsKeys.includes(key as mainRequests);
  }

  private isProviderFetchRequest(key: string) {
    return providerExtensionKeys.includes(key as providerFetchRequests);
  }

  private registerListeners() {
    process.setMaxListeners(11);

    process.on("exit", () => this.mainRequestHandler.killSelf());
    process.on("SIGQUIT", () => this.mainRequestHandler.killSelf());
    process.on("SIGINT", () => this.mainRequestHandler.killSelf());
    process.on("SIGUSR1", () => this.mainRequestHandler.killSelf());
    process.on("SIGUSR2", () => this.mainRequestHandler.killSelf());
    process.on("SIGHUP", () => this.mainRequestHandler.killSelf());
    process.on("uncaughtException", (e) => {
      console.error("Asynchronous error caught.", e.message ?? e.toString());
      if (e.message === "Channel closed") {
        process.exit();
      }
    });
  }

  public async parseMessage(message: extensionHostMessage) {
    await this.extensionHandler.initialized;
    if (this.isExtensionEvent(message.type as keyof MoosyncExtensionTemplate)) {
      this.extensionHandler.sendEvent(message as extensionEventMessage);
      return;
    }

    if (this.isMainRequest(message.type)) {
      return this.mainRequestHandler.parseRequest(
        message as mainRequestMessage,
      );
    }

    if (this.isProviderFetchRequest(message.type)) {
      return this.mainRequestHandler.parseProviderRequest(
        message as providerFetchRequestMessage,
      );
    }
  }
}

class MainRequestHandler {
  public handler: ExtensionHandler;

  constructor(handler: ExtensionHandler) {
    this.handler = handler;
  }

  public async parseProviderRequest(message: providerFetchRequestMessage) {
    return this.handler.handleProviderRequests(
      message.type,
      message.data.packageName,
    );
  }

  public async parseRequest(message: mainRequestMessage) {
    console.debug("Received message from main process", message.type);

    if (message.type === "findNewExtensions") {
      await this.handler.registerPlugins();
      await this.handler.startAll();
      return;
    }

    if (message.type === "getInstalledExtensions") {
      return this.handler.getInstalledExtensions();
    }

    if (message.type === "getExtensionIcon") {
      return this.handler.getExtensionIcon(message.data.packageName);
    }

    if (message.type === "toggleExtensionStatus") {
      this.handler.toggleExtStatus(
        message.data.packageName,
        message.data.enabled as boolean,
      );
      return;
    }

    if (message.type === "removeExtension") {
      return this.handler.removeExt(message.data.packageName);
    }

    if (message.type === "stopProcess") {
      this.killSelf(message.channel);
      return;
    }

    if (message.type === "extraExtensionEvents") {
      return this.handler.sendExtraEventToExtensions(
        message.data as unknown as ExtraExtensionEvents<ExtraExtensionEventTypes>,
      );
    }

    if (message.type === "getExtensionContextMenu") {
      const items = this.handler.getExtensionContextMenu(
        message.data.type as ContextMenuTypes,
      );
      return items;
    }

    if (message.type === "onClickedContextMenu") {
      this.handler.fireContextMenuCallback(
        message.data.id as string,
        message.data.packageName,
        message.data.arg as
          | Song
          | Artists
          | Album
          | Song[]
          | Playlist
          | undefined,
      );

      return;
    }

    if (message.type === "set-log-level") {
      // setLogLevel(message.data.level as LogLevelDesc);

      return;
    }

    if (message.type === "getAccounts") {
      const items = this.handler.getExtensionAccounts(
        message.data?.packageName,
      );
      return items;
    }

    if (message.type === "performAccountLogin") {
      return this.handler.performExtensionAccountLogin(
        message.data.packageName,
        message.data.accountId as string,
        message.data.loginStatus as boolean,
      );
    }

    if (message.type === "getDisplayName") {
      const name = this.handler.getDisplayName(message.data.packageName);
      return name;
    }
  }

  public killSelf(_?: string) {
    this.handler.stopAllExtensions().then(() => {
      // if (channel) this.sendToMain(channel);
      process.exit();
    });
  }
}
