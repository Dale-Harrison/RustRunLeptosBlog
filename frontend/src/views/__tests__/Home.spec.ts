import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import Home from '../Home.vue'

// Mock fetch globally
const fetchMock = vi.fn()
global.fetch = fetchMock

describe('Home.vue', () => {
    beforeEach(() => {
        fetchMock.mockReset()
    })

    it('renders loading state initially', () => {
        fetchMock.mockResolvedValue({
            ok: true,
            json: () => Promise.resolve({ message: 'Hello' }),
        })

        const wrapper = mount(Home, {
            global: {
                stubs: ['router-link']
            }
        })

        expect(wrapper.text()).toContain('Loading...')
    })

    it('renders message from API', async () => {
        // Mock API responses
        fetchMock
            .mockResolvedValueOnce({ // /api/hello
                ok: true,
                json: () => Promise.resolve({ message: 'Hello from Test' }),
            })
            .mockResolvedValueOnce({ // /auth/me
                ok: true,
                json: () => Promise.resolve(null),
            })
            .mockResolvedValueOnce({ // /api/posts
                ok: true,
                json: () => Promise.resolve([]),
            })

        const wrapper = mount(Home, {
            global: {
                stubs: ['router-link']
            }
        })

        // Wait for promises to resolve
        await flushPromises()

        expect(wrapper.text()).toContain('Hello from Test')
    })

    it('renders blog posts', async () => {
        const posts = [
            {
                id: 1,
                title: 'Test Post',
                content: 'Test Content',
                created_at: new Date().toISOString()
            }
        ]

        fetchMock
            .mockResolvedValueOnce({ // /api/hello
                ok: true,
                json: () => Promise.resolve({ message: 'Hello' }),
            })
            .mockResolvedValueOnce({ // /auth/me
                ok: true,
                json: () => Promise.resolve(null),
            })
            .mockResolvedValueOnce({ // /api/posts
                ok: true,
                json: () => Promise.resolve(posts),
            })

        const wrapper = mount(Home, {
            global: {
                stubs: ['router-link']
            }
        })

        await flushPromises()

        expect(wrapper.text()).toContain('Test Post')
        expect(wrapper.text()).toContain('Test Content')
    })
})
