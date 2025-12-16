import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
    // Simulate ramping up traffic
    stages: [
        { duration: '10s', target: 20 }, // Wrap up to 20 users
        { duration: '30s', target: 20 }, // Stay at 20 users
        { duration: '10s', target: 0 },  // Ramp down
    ],
};

export default function () {
    const res = http.get('https://inthedustyclocklesshours.balquidderocklabs.com/api/posts');

    check(res, {
        'status is 200': (r) => r.status === 200,
        'response time < 500ms': (r) => r.timings.duration < 500,
    });

    sleep(1);
}
