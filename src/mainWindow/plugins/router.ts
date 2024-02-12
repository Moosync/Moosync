/* eslint-disable */

/*
 *  router.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import {
	RouteRecordRaw,
	createRouter,
	createWebHashHistory,
	createWebHistory,
} from "vue-router";

const routes: RouteRecordRaw[] = [
	{
		name: "index",
		path: "",
		component: () => import("@/mainWindow/pages/index.vue"),
	},
	{
		name: "albums",
		path: "albums",
		component: () => import("@/mainWindow/pages/albums/index.vue"),
	},
	{
		name: "artists",
		path: "artists",
		component: () => import("@/mainWindow/pages/artists/index.vue"),
	},
	{
		name: "genre",
		path: "genre",
		component: () => import("@/mainWindow/pages/genre/index.vue"),
	},
	{
		name: "playlists",
		path: "playlists",
		component: () => import("@/mainWindow/pages/playlists/index.vue"),
	},
	{
		name: "recommendations",
		path: "recommendations",
		component: () => import("@/mainWindow/pages/recommendations/index.vue"),
	},
	{
		name: "search",
		path: "search",
		component: () => import("@/mainWindow/pages/search/index.vue"),
	},
	{
		name: "songs",
		path: "songs",
		component: () => import("@/mainWindow/pages/songs/index.vue"),
	},
	{
		name: "albums-single",
		path: "albums/single",
		props: true,
		component: () => import("@/mainWindow/pages/albums/single.vue"),
	},
	{
		name: "artists-single",
		path: "artists/single",
		props: true,
		component: () => import("@/mainWindow/pages/artists/single.vue"),
	},
	{
		name: "genre-single",
		path: "genre/single",
		props: true,
		component: () => import("@/mainWindow/pages/genre/single.vue"),
	},
	{
		name: "playlists-single",
		path: "playlists/single",
		props: true,
		component: () => import("@/mainWindow/pages/playlists/single.vue"),
	},
];

export const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{
			path: "/",
			component: () => import("@/mainWindow/layouts/default.vue"),
			children: routes,
		},
	],
});
