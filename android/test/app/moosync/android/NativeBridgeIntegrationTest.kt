package app.moosync.android

import android.content.Intent
import android.util.Log
import org.junit.AfterClass
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.BeforeClass
import org.junit.Test
import org.mockito.MockedStatic
import org.mockito.Mockito

class NativeBridgeIntegrationTest {

    companion object {
        private var mockedLog: MockedStatic<Log>? = null

        @JvmStatic
        @BeforeClass
        fun setUp() {
            mockedLog = Mockito.mockStatic(Log::class.java)
        }

        @JvmStatic
        @AfterClass
        fun tearDown() {
            mockedLog?.close()
        }
    }

    @Test
    fun testNativeBridgeObjectInitializes() {
        NativeBridge.init()

        assertNotNull(NativeBridge)
    }

    @Test
    fun testNativeBridgeBackPressFallback() {
        var fallbackInvoked = false
        val handler = BackPressHandler(onUnhandled = { fallbackInvoked = true })

        handler.onBackPressed()

        assertTrue(fallbackInvoked)
    }

    @Test
    fun testNativeBridgeDeepLinkHandling() {
        val handler = DeepLinkHandler()
        val intent = Mockito.mock(Intent::class.java)
        Mockito.`when`(intent.action).thenReturn(Intent.ACTION_VIEW)
        Mockito.`when`(intent.dataString).thenReturn("moosync://oauth/callback?code=native_token")

        handler.handleIntent(intent)

        assertEquals("moosync://oauth/callback?code=native_token", handler.pendingDeepLink)
    }

    @Test
    fun testNativeBridgeDeepLinkProcessing() {
        val handler = DeepLinkHandler()
        val intent = Mockito.mock(Intent::class.java)
        Mockito.`when`(intent.action).thenReturn(Intent.ACTION_VIEW)
        Mockito.`when`(intent.dataString).thenReturn("moosync://oauth/callback?code=native_token")
        handler.handleIntent(intent)

        handler.processPendingDeepLink()

        assertNull(handler.pendingDeepLink)
    }
}
