/*
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider';
 *  providers.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

interface Recommendations {
  songs: Song[]
}

interface Provider {
  username: string | undefined
  provider: import('@/utils/ui/providers/generics/genericProvider').GenericProvider
  key: string
}
