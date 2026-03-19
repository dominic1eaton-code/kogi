import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioBinderComponent } from './portfolio-binder.component';

describe('PortfolioBinderComponent', () => {
  let component: PortfolioBinderComponent;
  let fixture: ComponentFixture<PortfolioBinderComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioBinderComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioBinderComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
